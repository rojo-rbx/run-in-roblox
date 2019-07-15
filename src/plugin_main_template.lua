local modeValue = game:FindFirstChild("RUN_IN_ROBLOX_MODE")

while modeValue == nil do
	game.ChildAdded:Wait()
	modeValue = game:FindFirstChild("RUN_IN_ROBLOX_MODE")
end

if modeValue.ClassName ~= "StringValue" then
	warn("run-in-roblox found RUN_IN_ROBLOX_MODE marker, but it was the wrong type.")
	return
end

local mode = modeValue.Value

local HttpService = game:GetService("HttpService")
local LogService = game:GetService("LogService")
local RunService = game:GetService("RunService")

local PORT = {{PORT}}
local SERVER_URL = ("http://localhost:%d"):format(PORT)

local queuedMessages = {}
local timeSinceLastSend = 0
local messageSendRate = 0.2
local closeDelay = 0.5
local running = false

local heartbeatConnection = RunService.Heartbeat:Connect(function(dt)
	timeSinceLastSend = timeSinceLastSend + dt

	if timeSinceLastSend >= messageSendRate and running then
		local encoded = HttpService:JSONEncode(queuedMessages)
		queuedMessages = {}
		timeSinceLastSend = 0

		HttpService:PostAsync(SERVER_URL .. "/messages", encoded)
	end
end)

local logTypeToLevel = {
	[Enum.MessageType.MessageOutput] = "Print",
	[Enum.MessageType.MessageInfo] = "Info",
	[Enum.MessageType.MessageWarning] = "Warning",
	[Enum.MessageType.MessageError] = "Error",
}

local logConnection = LogService.MessageOut:Connect(function(body, messageType)
	table.insert(queuedMessages, {
		type = "Output",
		level = logTypeToLevel[messageType] or "Info",
		body = body,
	})
end)

HttpService:PostAsync(SERVER_URL .. "/start", "")

running = true

spawn(function()
	local success, errorMessage = pcall(function()
		require(script.main)
	end)

	if not success then
		warn("main encountered an error:")
		warn(errorMessage)
	end

	wait(closeDelay)
	running = false
end)

local timeout = tick() + {{TIMEOUT}}

while running and tick() < timeout do
	wait(1)
end

local success, errorMessage = pcall(function()
	HttpService:PostAsync(SERVER_URL .. "/stop", "")
end)

if not success then
	warn("Could not send POST request to stop")
	warn(errorMessage)
end

heartbeatConnection:Disconnect()
logConnection:Disconnect()
