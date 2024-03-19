package coinbase

import (
	"errors"
	"strings"
	"transactions/structs"
)

type Provider struct {
	baseUrl string
	key string
	secret string
}

func (provider Provider) Swap(account structs.Account, transaction structs.TransactionInfo) (*structs.TransactionResult, error) {
	// @TODO
	return new(structs.TransactionResult), nil
}

func Verify() (ok bool) {
	// @TODO
	return
}

func verifyAuthData(key string, secret string) error {
	// @TODO
	return nil
}

func NewProvider(authData string) (*Provider, error)  {
	splitAuthData := strings.Split(authData, ":")
	if len(splitAuthData) != 2 {
		return nil, errors.New("invalid auth data format")
	}

	key := splitAuthData[0]
	secret := splitAuthData[1]

	provider := Provider {
		baseUrl: "@TODO",
		key: key,
		secret: secret,
	}

	err := verifyAuthData(key, secret)
	if err != nil {
		return nil, err
	}

	return &provider, nil
}
