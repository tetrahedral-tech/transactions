package void

import (
	"encoding/hex"
	"fmt"
	"time"
	"transactions/structs"
)

type Provider struct {}

func (provider Provider) Swap(account structs.Account, transaction structs.TransactionInfo) (*structs.TransactionResult, error) {
	now := time.Now()

	return &structs.TransactionResult{
		Id: hex.EncodeToString([]byte(fmt.Sprint(now.Unix()))),
		Time: now,
	}, nil
}

func Verify() (ok bool) {
	ok = true
	return
}

func NewProvider() *Provider  {
	return new(Provider)
}
