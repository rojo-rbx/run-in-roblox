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

local heartbeatConnection = RunService.Heartbeat:Connect(function(dt)
	timeSinceLastSend = timeSinceLastSend + dt

	if timeSinceLastSend >= messageSendRate then
		local encoded = HttpService:JSONEncode(queuedMessages)
		queuedMessages = {}
		timeSinceLastSend = 0

		HttpService:PostAsync(SERVER_URL, "/messages", encoded)
	end
end)

local logTypeToLevel = {
	[Enum.MessageType.MessageOutput] = "Output",
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

HttpService:PostAsync(SERVER_URL .. "/start", "hi")

require(script.Parent.Main)

HttpService:PostAsync(SERVER_URL .. "/finish", "hi")

heartbeatConnection:Disconnect()
logConnection:Disconnect()