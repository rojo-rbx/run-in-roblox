local PORT = "{{PORT}}"
local SERVER_ID = "{{SERVER_ID}}"

local SERVER_URL = string.format("http://localhost:%s", PORT)

local HttpService = game:GetService("HttpService")
local RunService = game:GetService("RunService")

local pingSuccess, remoteServerId =
	pcall(
	function()
		return HttpService:GetAsync(SERVER_URL)
	end
)

-- If there was a transport error, just abort silently.
--
-- It's possible that the run-in-roblox plugin is erroneously installed, and we
-- should minimize our impact to the user.
if not pingSuccess then
	return
end

-- There is a server running on that port, but it isn't the right run-in-roblox
-- server and might be some other HTTP server.
if remoteServerId ~= SERVER_ID then
	return
end

local queuedMessages = {}
local timeSinceLastSend = 0
local messageSendRate = 0.1

local function flushMessages()
	if #queuedMessages == 0 then
		return
	end

	local encoded = HttpService:JSONEncode(queuedMessages)
	queuedMessages = {}

	timeSinceLastSend = 0
	HttpService:PostAsync(SERVER_URL .. "/messages", encoded)
end

local function logMessage(body, level)
	table.insert(
		queuedMessages,
		{
			type = "Output",
			level = level,
			body = body
		}
	)
end

local function formatMessageBody(...)
	local arg = {...}

	local body = ""

	for index, value in ipairs(arg) do
		body = body .. tostring(value)
		if index ~= #arg then
			body = body .. " "
		end
	end

	return body
end

local function setupEnv(mainFunc)
	local currentEnv = getfenv(mainFunc)

	-- We aren't listening to LogService because it could contain logs from elsewhere.
	-- Instead, we're stubbing print and warn and catching errors in runScript()

	local newEnv = {
		print = function(...)
			local body = formatMessageBody(...)
			logMessage(body, "Print")
		end,
		warn = function(...)
			local body = formatMessageBody(...)
			logMessage(body, "Warning")
		end
	}

	setmetatable(newEnv, {__index = currentEnv})

	return setfenv(mainFunc, newEnv)
end

local function runScript()
	local requireSuccess, requireResult = xpcall(require, debug.traceback, script.Main)

	if requireSuccess then
		local sandboxedMainFunc = setupEnv(requireResult)
		local runSuccess, runResult = xpcall(sandboxedMainFunc, debug.traceback)

		if not runSuccess then
			logMessage(runResult, "Error")
		end
	else
		logMessage(requireResult, "Error")
	end
end

local heartbeatConnection =
	RunService.Heartbeat:Connect(
	function(dt)
		timeSinceLastSend = timeSinceLastSend + dt

		if timeSinceLastSend >= messageSendRate then
			flushMessages()
		end
	end
)

HttpService:PostAsync(SERVER_URL .. "/start", "")

runScript()
heartbeatConnection:Disconnect()
flushMessages()

HttpService:PostAsync(SERVER_URL .. "/stop", "")
