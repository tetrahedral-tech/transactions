package void

import (
	"encoding/hex"
	"fmt"
	"time"
	"transactions/structs"
)

type Provider struct{}
type verification bool

func (provider Provider) Swap(account structs.Account, transaction structs.TransactionInfo) (*structs.TransactionResult, error) {
	now := time.Now()

	err := provider.Verify(true)
	if err != nil {
		return nil, err
	}

	return &structs.TransactionResult{
		Id:   hex.EncodeToString([]byte(fmt.Sprint(now.Unix()))),
		Time: now,
	}, nil
}

func (provider Provider) PairSupported(pair structs.Pair) (ok bool) {
	ok = true
	return
}

func (provider Provider) Verify(dataInterface interface{}) error {
	data, ok := dataInterface.(verification)
	if !ok {
		return fmt.Errorf("verification data decoded incorrectly: %v", data)
	}

	if !data {
		return fmt.Errorf("verification data was %t", data)
	}

	return nil
}

func NewProvider() *Provider {
	return new(Provider)
}
