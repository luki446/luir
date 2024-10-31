
function Fib(n)
    if n < 2 then
        return 1
    else
        return Fib(n - 1) + Fib(n - 2)
    end
end

-- example comment
print(Fib(25))

num = 10

repeat
  print("Hello", num)
  num = num - 1
until num < 0
