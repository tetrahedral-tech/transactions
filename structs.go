package main

import (
	provider "transactions/provider"

	"go.mongodb.org/mongo-driver/bson/primitive"
)

type TradeType int8

const (
	Buy       TradeType = 0
	Sell      TradeType = 1
	NoAction  TradeType = 2
)

type Coin struct {
	Name string
	supportedProviders []provider.TradeProvider
}

type Pair struct {
	A Coin
	B Coin
}

type AlgorithmSignal struct {
	Algorithm string `json:"algorithm"`
	Amount    int    `json:"amount"`
	Signal    string `json:"signal"`
}

type Status struct {
	Name string `bson:"name"`
	Time int    `bson:"time"`
}

type Account struct {
	ID                  primitive.ObjectID `bson:"_id"`
	Algorithm           primitive.ObjectID `bson:"algorithm"`
	EncryptedPrivateKey string             `bson:"encryptedPrivateKey"`
	Pair                []string           `bson:"pair"`
	Provider            string             `bson:"provider"`
	Interval            int                `bson:"interval"`
}
