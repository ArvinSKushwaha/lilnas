package main

import (
	"C"
	"fmt"
	"github.com/fsnotify/fsnotify"
	"log"
	"sync"
)

//export NewWaitGroup
func NewWaitGroup() *sync.WaitGroup {
	return &sync.WaitGroup{}
}

//export Nas
func Nas(wg *sync.WaitGroup) {
	fmt.Println("Hello, world!")
	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		log.Panicln(err)
	}
	err = watcher.Add("/home/arvinsk/Documents/CodeProjects/lilnas/lilnas_server/go")
	if err != nil {
		log.Panicln(err)
	}

	wg.Add(1)
	go handleEvent(watcher.Events, watcher.Errors, wg)
	wg.Wait()
}

func handleEvent(ch1 chan fsnotify.Event, ch2 chan error, wg *sync.WaitGroup) {
	defer wg.Done()
	for {
		select {
		case event, ok := <-ch1:
			if !ok {
				return
			}

			log.Println(event)
		case err, ok := <-ch2:
			if !ok {
				return
			}

			log.Println(err)
		}
	}
}

func main() {
	wg := NewWaitGroup()
	Nas(wg)
}
