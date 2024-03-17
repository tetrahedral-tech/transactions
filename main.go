package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type TradeSignal struct {
	Algorithm string `json:"algorithm"`
	Amount	int	`json:"amount"`
	Signal	string `json:"signal"`
}

func getSignals(pair string, interval int16) ([]TradeSignal, error) {
	url := fmt.Sprintf("http://127.0.0.1:5000/signals?pair=%s&interval=%d", pair, interval)

	response, err := http.Get(url)
	if err != nil {
		return nil, fmt.Errorf("error making HTTP request: %v", err)
	}
	defer response.Body.Close()

	responseBody, err := io.ReadAll(response.Body)
	if err != nil {
		return nil, fmt.Errorf("error reading response body: %v", err)
	}

	var parsedResponses []TradeSignal
	err = json.Unmarshal(responseBody, &parsedResponses)
	if err != nil {
		return nil, fmt.Errorf("error parsing JSON: %v", err)
	}

	return parsedResponses, nil
}

func priceUpdateHandler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprint(w, "Updating signals!")
	fmt.Println("Price Update on timestamp:", time.Now().Unix())

}

func main() {
	http.HandleFunc("/price_update", priceUpdateHandler)

	fmt.Println("Server listening on http://localhost:8080")
	http.ListenAndServe(":8080", nil)
}
