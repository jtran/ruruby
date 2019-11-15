class Vec
    def initialize(x,y)
        @x=x;@y=y
    end
    def add(v)
        Vec.new(@x + v.x, @y + v.y)
    end
    def x; @x; end
    def y; @y; end
end

class Vec
    def initialize(x,y)
        @x=x
        @y=y
    end
    def x
        @x
    end
    def y
        @y
    end
    def +(other)
        Vec.new(@x + other.x, @y + other.y)
    end
    def -(other)
        Vec.new(@x - other.x, @y - other.y)
    end
    def *(other)
        Vec.new(@x * other.x, @y * other.y)
    end
end

v1 = Vec.new(3,4)
v2 = Vec.new(6,14)
v = v1 + v2
puts("(#{v.x}, #{v.y})")
v = v1 - v2
puts("(#{v.x}, #{v.y})")
v = v1 * v2
puts("(#{v.x}, #{v.y})")
