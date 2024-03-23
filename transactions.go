package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"sync"
	"time"
	"transactions/structs"

	"go.mongodb.org/mongo-driver/mongo"
)

type connectionWaitTimeouts struct {
	main  time.Duration
	retry time.Duration
}

func waitForConnection(url string, timeouts connectionWaitTimeouts) bool {
	var wg sync.WaitGroup
	wg.Add(1)

	success := make(chan bool, 1)

	go func() {
		go func() {
			defer wg.Done()
			for {
				_, err := http.Get(url)
				if err == nil {
					break
				}
				time.Sleep(timeouts.retry)
			}
		}()
		wg.Wait()
		success <- true
	}()

	select {
	case <-success:
		return true
	case <-time.After(timeouts.main):
		return false
	}
}

func runTransactions(database mongo.Database) error {
	cursor, err := getAccountCursor(database)
	if err != nil {
		return err
	}

	algorithmIdToName, err := getAlgorithmIdToNameMap(database)
	if err != nil {
		return err
	}

	router := exec.Command("node", "transaction-router/")
	router.Stderr = os.Stderr
	router.Stdin = os.Stdin

	defer func() {
		process := router.Process
		if process != nil {
			err := process.Kill()
			if err != nil {
				fmt.Printf("error killing ")
			}
		}
	}()
	err = router.Start()
	if err != nil {
		return err
	}

	ok := waitForConnection("http://localhost:6278/ping", connectionWaitTimeouts{
		30 * time.Second,
		500 * time.Millisecond,
	})

	if !ok {
		return fmt.Errorf("connection to transaction router failed")
	}

	buildTransaction := func(account structs.Account, algorithmSignals map[string]structs.AlgorithmSignal) (*structs.TransactionInfo, error) {
		algorithmName, ok := algorithmIdToName[account.Algorithm]
		if !ok {
			return nil, fmt.Errorf("algorithm not found in algorithm to name map: %v", account.Algorithm)
		}

		signal, ok := algorithmSignals[algorithmName]
		if !ok {
			return nil, fmt.Errorf("algorithm name not found in algorithm to signal map: %v", algorithmName)
		}

		if signal.Signal == structs.NoAction {
			// return nil, fmt.Errorf("signal is no action: %v %v", account, signal)
		}

		return &structs.TransactionInfo{
			Amount:   signal.Amount * 10,
			Action:   signal.Signal,
			Pair:     account.Pair,
			Provider: account.Provider,
		}, nil
	}

	// @TODO parallelize this
	for cursor.Next(context.Background()) {
		var account structs.Account
		err := cursor.Decode(&account)
		if err != nil {
			fmt.Printf("error decoding account: %v\n", err)
			continue
		}

		algorithmSignals, err := getSignals(account.Pair, account.Interval)
		if err != nil {
			fmt.Printf("error getting signals: %v\n", err)
			continue
		}

		transaction, err := buildTransaction(account, algorithmSignals)
		if err != nil {
			fmt.Printf("error building transaction: %v\n", err)
			continue
		}

		marshalledTransaction, err := json.Marshal(transaction)
		if err != nil {
			fmt.Printf("error unmarshalling transaction: %v\n", err)
			continue
		}

		routerUri, ok := os.LookupEnv("TRANSACTION_ROUTER_URI")
		if !ok {
			panic("DB_URI is not in .env")
		}

		swap, err := http.Post(fmt.Sprintf("%s/route", routerUri), "application/json", bytes.NewBuffer(marshalledTransaction))
		if err != nil {
			fmt.Printf("error sending transaction to transaction router: %v\n", err)
			continue
		}

		fmt.Printf("transaction sent: %v %v\n", swap, account)
	}

	if err := cursor.Err(); err != nil {
		return err
	}

	return nil
}
