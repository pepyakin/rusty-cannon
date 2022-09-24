package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"testing"
	"time"
	uc "github.com/unicorn-engine/unicorn/bindings/go/unicorn"
)

func TestMinigethUnicorn(t *testing.T) {
	uniram := make(map[uint32](uint32))
	RunUnicorn("../mipigo/arbitrary-prover-main.bin", uniram, true, func(step int, mu uc.Unicorn, ram map[uint32](uint32)) {
		SyncRegs(mu, ram)
		if step%1 == 0 {
			steps_per_sec := float64(step) * 1e9 / float64(time.Now().Sub(ministart).Nanoseconds())

			pc, _ := mu.RegRead(uc.MIPS_REG_PC)
			gp, _ := mu.RegRead(uc.MIPS_REG_ZERO + 28)
			ra, _ := mu.RegRead(uc.MIPS_REG_RA)
			fmt.Printf(
				"%10d pc: %x gp: %x ra: %x steps per s %f ram entries %d\n", 
				step, pc, gp, ra, steps_per_sec, len(ram))
		}
	})
}

func TestSimpleEVM(t *testing.T) {
	files, err := ioutil.ReadDir("test/bin")
	if err != nil {
		log.Fatal(err)
	}
	good := true
	gas := uint64(0)
	for _, f := range files {
		ram := make(map[uint32](uint32))
		ram[0xC000007C] = 0x5EAD0000
		fn := "test/bin/" + f.Name()
		LoadMappedFile(fn, ram, 0)

		start := time.Now()
		remainingGas, err := RunWithRam(ram, 100, 0, "testoracle/", nil)
		elapsed := time.Now().Sub(start)

		fmt.Println(err, remainingGas, elapsed,
			ram[0xbffffff4], ram[0xbffffff8], fmt.Sprintf("%x", ram[0xc0000080]), fn)
		if err != nil {
			log.Fatal(err)
		}
		good = good && ((ram[0xbffffff4] & ram[0xbffffff8]) == 1)
		gas += remainingGas
	}
	if !good {
		panic("some tests failed")
	}
	fmt.Println("used", gas, "gas")
}
