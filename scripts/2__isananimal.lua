-- when({"$ $ is a $animal", "$ $animal is red"}, function (results)
local testval = 0

when({"$ $animal is red"}, function (results)
    print("HELLO FROM LUA")
    print(results)
    print(#results)
    -- retract("#2 %")
    for index, result in ipairs(results) do
        -- claim("red animal seen")
        print("I see a: "..result["animal"])
        testval = testval + 1
        print(testval)
    end
end)