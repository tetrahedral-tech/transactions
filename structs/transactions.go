package structs

import (
	"bytes"
	"encoding/json"
	"time"
)

type TradeType int8

const (
	Buy       TradeType = 0
	Sell      TradeType = 1
	NoAction  TradeType = 2
)

type TransactionInfo struct {
	Amount float64
	Action TradeType
	Pair   Pair
}

type TransactionResult struct {
	Id   string
	Time time.Time
}

// JSON marshaling
func (s TradeType) String() string {
	return toString[s]
}

var toString = map[TradeType]string{
	Buy: "buy",
	Sell: "sell",
	NoAction: "no_action",
}

var toID = map[string]TradeType{
	"buy": Buy,
	"sell": Sell,
	"no_action": NoAction,
}

// MarshalJSON marshals the enum as a quoted json string
func (s TradeType) MarshalJSON() ([]byte, error) {
	buffer := bytes.NewBufferString(`"`)
	buffer.WriteString(toString[s])
	buffer.WriteString(`"`)
	return buffer.Bytes(), nil
}

// UnmarshalJSON unmashals a quoted json string to the enum value
func (s *TradeType) UnmarshalJSON(b []byte) error {
	var j string
	err := json.Unmarshal(b, &j)
	if err != nil {
		return err
	}
	// Note that if the string cannot be found then it will be set to the zero value, 'Buy' in this case.
	*s = toID[j]
	return nil
}