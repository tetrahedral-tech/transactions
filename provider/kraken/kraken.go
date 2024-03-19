package provider

import (
	"errors"
	"fmt"
	"log"
	"strconv"
	"strings"

	krakenapi "github.com/beldur/kraken-go-api-client"
)

type Provider struct {
	key    string
	secret string
}

func Swap(apiKey string, apiSecret string, tradeType string, pair string, price int64) {

	api := krakenapi.New(apiKey, apiSecret)
	args := map[string]string{
		"price": strconv.Itoa(int(price)),
	}
	response, err := api.AddOrder(pair, tradeType, "limit", "1", args)

	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Kraken api Response: %+v\n", response)

}

func Verify() bool {
	return true
}

func verifyAuthData(key string, secret string) error {
	api := krakenapi.New(key, secret)
	balance, err := api.Balance()

	if err != nil {
		return err
	}

	// Check balance things later (:, for now this verifies the tokens
	println(balance)
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
		key:    key,
		secret: secret,
	}

	err := verifyAuthData(key, secret)
	if err != nil {
		return nil, err
	}

	return &provider, nil
}
