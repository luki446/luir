
function Fib(n)
    if n < 2 then
        return 1
    else
        return Fib(n - 1) + Fib(n - 2)
    end
end

print(Fib(25))
