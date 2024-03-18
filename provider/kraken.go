package provider

import (
	"fmt"
	"log"
	"strconv"

	krakenapi "github.com/beldur/kraken-go-api-client"
)

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
