claim("wish \"http://192.168.1.34:8000/smiley.png\" would be thermal printed")
cache = {}

function draw(t)
    local ill = Illumination.new()
    ill:text{x=0, y=50, text=t}
    claim("wish you had graphics", {"", tostring(ill)})
end

when({"$ clock time is $t"}, function (results)
    -- print("clock "..sub_id)
    retract("#3 time is %")
    retract("#3 wish you had graphics %")
    for index, result in ipairs(results) do
        claim("time is "..os.time())
        draw("time is "..os.time())
    end
end)

when({"$ fps is $fps"}, function (results)
    -- print("fps "..sub_id)
    retract("#3 wish window had graphics %")
    for index, result in ipairs(results) do
        table.insert(cache, result["fps"])
        if #cache > 10 then
            table.remove(cache, 1)
        end
        local t = ""
        for index, c in ipairs(cache) do
            t = t .. "\n" .. c
        end
        local ill = Illumination.new()
        ill:text{x=0, y=100, text=t, size=60}
        claim("wish window had graphics", {"", tostring(ill)})
    end
end)
