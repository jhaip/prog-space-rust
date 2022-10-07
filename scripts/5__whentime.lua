-- when({"$ time is $time"}, function (results)
--     retract("#5 %")
--     for index, result in ipairs(results) do
--         claim("I heard the time is "..results["time"])
--         local ill = Illumination.new()
--         ill:text{x=0, y=40, text="heard:"..result["time"]}
--         claim("wish you had graphics ", {"", tostring(ill)})
--     end
-- end)
when({"$ ble $ says button is pressed"}, function (results)
    retract("#5 %")
    for index, result in ipairs(results) do
        local ill = Illumination.new()
        ill:text{x=0, y=40, text="BUTTON PRESSED", size=100}
        claim("wish you had graphics ", {"", tostring(ill)})
    end
end)