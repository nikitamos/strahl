{-# LANGUAGE MultiParamTypeClasses #-}
{-# LANGUAGE FunctionalDependencies #-}

class Expr x where
    eval :: x -> Int

type Print = Show

class Expr x => ExprAlgebra x where
  lit :: Int -> x
  add :: x -> x -> x

data SimpleAlgebra
  = SimpleLit Int
  | Sum (SimpleAlgebra) (SimpleAlgebra)

instance Expr SimpleAlgebra where
  eval x = case x of
    SimpleLit n -> n
    Sum u v -> eval u + eval v

instance ExprAlgebra SimpleAlgebra where
  lit = SimpleLit
  add = Sum

type Vec3 = (Float, Float, Float)
data AllNodes = Sphere Vec3 Float | Plane Vec3

class NodeAlgebra b where
  sphere :: Vec3 -> Float -> b
  plane :: Vec3 -> b

-- class Node

main =
    let x = Sum (SimpleLit 1) (SimpleLit 3)
    in print (eval x)
