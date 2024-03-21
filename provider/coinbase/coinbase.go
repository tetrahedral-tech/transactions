package coinbase

import (
	"errors"
	"fmt"
	"strings"
	"transactions/structs"
)

type Provider struct {
	baseUrl string
	key     string
	secret  string
}
type verification struct{}

func (provider Provider) Swap(account structs.Account, transaction structs.TransactionInfo) (*structs.TransactionResult, error) {
	// @TODO
	return new(structs.TransactionResult), nil
}

func (provider Provider) PairSupported(pair structs.Pair) (ok bool) {
	// @TODO
	return
}

func (provider Provider) Verify(dataInterface interface{}) error {
	data, ok := dataInterface.(verification)
	if !ok {
		return fmt.Errorf("verification data decoded incorrectly: %v", data)
	}

	// @TODO

	return nil
}

func verifyAuthData(key string, secret string) error {
	// @TODO
	return nil
}

func NewProvider(authData string) (*Provider, error) {
	splitAuthData := strings.Split(authData, ":")
	if len(splitAuthData) != 2 {
		return nil, errors.New("invalid auth data format")
	}

	key := splitAuthData[0]
	secret := splitAuthData[1]

	provider := Provider{
		baseUrl: "@TODO",
		key:     key,
		secret:  secret,
	}

	err := verifyAuthData(key, secret)
	if err != nil {
		return nil, err
	}

	return &provider, nil
}
