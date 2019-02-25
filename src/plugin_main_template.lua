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

local PORT = {{PORT}}
local SERVER_URL = ("http://localhost:%d"):format(PORT)

local function postMessage(text)
	HttpService:PostAsync(SERVER_URL .. "/message", text)
end

HttpService:PostAsync(SERVER_URL .. "/start", "")

require(script.Parent.Main)(postMessage)

HttpService:PostAsync(SERVER_URL .. "/finish", "")