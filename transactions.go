package main

import (
	"context"
	"fmt"
	"transactions/provider"
	"transactions/structs"

	"go.mongodb.org/mongo-driver/mongo"
)

func runTransactions(database mongo.Database) error {
	cursor, err := getAccountCursor(database)
	if err != nil {
		return err
	}

	algorithmIdToName, err := getAlgorithmIdToNameMap(database)
	if err != nil {
		return err
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

		return &structs.TransactionInfo {
			Amount: signal.Amount * 10,
			Action: signal.Signal,
			Pair: account.Pair,
		}, nil
	}

	for cursor.Next(context.Background()) {
		var account structs.Account
		err := cursor.Decode(&account)
		if err != nil {
			fmt.Printf("error decoding account: %v", err)
			continue
		}

		algorithmSignals, err := getSignals(account.Pair, account.Interval)
		if err != nil {
			fmt.Printf("error getting signals: %v", err)
			continue
		}

		provider, err := provider.BuildProvider(account.Provider, account.EncryptedPrivateKey)
		if err != nil {
			fmt.Printf("error building provider: %v", err)
			continue
		}
		
		transaction, err := buildTransaction(account, algorithmSignals)
		if err != nil {
			fmt.Printf("error building transaction: %v", err)
			continue
		}

		swap, err := (*provider).Swap(account, *transaction)
		if err != nil {
			fmt.Printf("error swapping: %v", err)
		}

		fmt.Printf("swap executed: %v", swap)
	}

	return nil
}