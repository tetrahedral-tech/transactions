package main

import "go.mongodb.org/mongo-driver/bson/primitive"

type TradeType int8

const (
	buy       TradeType = 0
	sell      TradeType = 1
	no_action TradeType = 2
)

type AlgorithmSignal struct {
	Algorithm string `json:"algorithm"`
	Amount    int    `json:"amount"`
	Signal    string `json:"signal"`
}

type Worth struct {
	ID        string  `bson:"_id"`
	Timestamp int     `bson:"timestamp"`
	Value     float64 `bson:"value"`
}

type Status struct {
	Name string `bson:"name"`
	Time int    `bson:"time"`
}

type Bot struct {
	ID                  primitive.ObjectID `bson:"_id"`
	Owner               primitive.ObjectID `bson:"owner"`
	Algorithm           primitive.ObjectID `bson:"algorithm"`
	StrengthToUsd       int                `bson:"strengthToUSD"`
	EncryptedPrivateKey string             `bson:"encryptedPrivateKey"`
	Worth               []Worth            `bson:"worth"`
	Status              Status             `bson:"status"`
	Pair                []string           `bson:"pair"`
	Provider            string             `bson:"provider"`
	Interval            int                `bson:"interval"`
}
