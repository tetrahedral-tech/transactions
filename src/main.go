package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"time"

	"github.com/joho/godotenv"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

type AlgorithmResponse struct {
	Algorithm string `json:"algorithm"`
	Amount    int    `json:"amount"`
	Signal    string `json:"signal"`
}

func fetchSignals(pair string, interval int16) ([]AlgorithmResponse, error) {
	url := fmt.Sprintf("http://127.0.0.1:5000/signals?pair=%s&interval=%d", pair, interval)

	response, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer response.Body.Close()

	responseBody, err := io.ReadAll(response.Body)
	if err != nil {
		return nil, err
	}

	var parsedResponses []AlgorithmResponse
	err = json.Unmarshal(responseBody, &parsedResponses)
	if err != nil {
		return nil, err
	}

	// if you need to debug ~~
	//	for _, resp := range parsedResponses {
	//		fmt.Println("Algorithm:", resp.Algorithm)
	//		fmt.Println("-	Amount:", resp.Amount)
	//	fmt.Println("-	Signal:", resp.Signal)
	//	}

	return parsedResponses, nil
}

func getBots() (*mongo.Cursor, error) {
	err := godotenv.Load(".env")
	if err != nil {
		return nil, err
	}

	dbURI := os.Getenv("DB_URI")

	clientOptions := options.Client().ApplyURI(dbURI)

	client, err := mongo.Connect(context.Background(), clientOptions)
	if err != nil {
		return nil, err
	}

	bots_collection := client.Database("database").Collection("bots")
	filter := bson.M{"status.name": "running"}

	cursor, err := bots_collection.Find(context.Background(), filter)
	if err != nil {
		return nil, err
	}
	defer cursor.Close(context.Background())

	return cursor, nil
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
