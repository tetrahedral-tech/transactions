package main

type TradeType int8

const (
	buy       TradeType = 0
	sell      TradeType = 1
	no_action TradeType = 2
)

type AlgorithmSignal struct {
	Algorithm string `json:"algorithm"`
	Amount    int `json:"amount"`
	Signal    string `json:"signal"`
}