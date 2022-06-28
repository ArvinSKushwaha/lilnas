package main

import (
    "C"
    "fmt"
)

//export Nas
func Nas() {
    fmt.Println("Hello, world!")
}

func main() {
    Nas()
}
