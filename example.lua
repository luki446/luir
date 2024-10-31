
function Fib(n)
    if n < 2 then
        return 1
    else
        return Fib(n - 1) + Fib(n - 2)
    end
end

-- example comment
print(Fib(25))
