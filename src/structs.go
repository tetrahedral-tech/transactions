package main

import "go.mongodb.org/mongo-driver/bson/primitive"

type TradeType int8

const (
	buy       TradeType = 0
	sell      TradeType = 1
	no_action TradeType = 2
)

type AlgorithmSignal struct {
	algorithm string `json:"algorithm"`
	amount    int    `json:"amount"`
	signal    string `json:"signal"`
}

type Worth struct {
	id        string  `bson:"_id"`
	timestamp int     `bson:"timestamp"`
	value     float64 `bson:"value"`
}

type Status struct {
	name string `bson:"name"`
	time int    `bson:"time"`
}

type Bot struct {
	id                  primitive.ObjectID `bson:"_id"`
	owner               primitive.ObjectID `bson:"owner"`
	algorithm           primitive.ObjectID `bson:"algorithm"`
	strengthToUsd       int                `bson:"strengthToUSD"`
	encryptedPrivateKey string             `bson:"encryptedPrivateKey"`
	worth               []Worth            `bson:"worth"`
	status              Status             `bson:"status"`
	pair                []string           `bson:"pair"`
	provider            string             `bson:"provider"`
	interval            int                `bson:"interval"`
}
