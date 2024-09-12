
function fac(n)
    if n <= 1 then
        return 1
    else
        return n * fac(n - 1)
    end
end

fac(2)