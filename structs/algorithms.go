package structs

type AlgorithmSignal struct {
	Algorithm string    `json:"algorithm"`
	Amount    float64   `json:"amount"`
	Signal    TradeType `json:"signal"`
}
