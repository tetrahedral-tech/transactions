package provider

import (
	"transactions/provider/coinbase"
	"transactions/provider/void"
	"transactions/structs"
)

type TradeProvider interface {
	Swap(account structs.Account, transaction structs.TransactionInfo) (*structs.TransactionResult, error)
	Verify(interface{}) error
	PairSupported(pair structs.Pair) bool
}

func BuildProvider(providerName string, auth string) (*TradeProvider, error) {
	var provider TradeProvider
	var err error

	switch providerName {
	case "coinbase":
		provider, err = coinbase.NewProvider(auth)
	case "void":
		provider = void.NewProvider()
	}

	if err != nil {
		return nil, err
	}

	return &provider, nil
}
