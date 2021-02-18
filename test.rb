class C
    def fn
        #@a = 100
        10000000.times do
            @a = 100
            @a = 100
            @a = 100
            @a = 100
            @a = 100
            @a = 100
            @a = 100
            @a = 100
            @a = 100
            @a = 100
        end
    end
end

o = C.new
o.fn