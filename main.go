package main

import (
	"context"
	"fmt"
	"net/http"
	"os"

	"github.com/joho/godotenv"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

func getAccountCursor(database mongo.Database) (*mongo.Cursor, error) {
	bots_collection := database.Collection("bots")
	filter := bson.M{"status.name": "running"}

	cursor, err := bots_collection.Find(context.Background(), filter)
	if err != nil {
		return nil, err
	}

	return cursor, nil
}

func priceUpdateHandlerWrapper(database mongo.Database) func(w http.ResponseWriter, r *http.Request) {
	return func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, "")
		fmt.Println("running transactions")
		err := runTransactions(database)
		fmt.Printf("error running transactions: %v", err)
	}
}

func main() {
	err := godotenv.Load(".env")
	if err != nil {
		panic(err)
	}

	dbUri, ok := os.LookupEnv("DB_URI")
	if !ok {
		panic("DB_URI is not in .env")
	}

	client, err := mongo.Connect(
		context.Background(),
		options.Client().ApplyURI(dbUri),
	)
	if err != nil {
		panic(err)
	}

	priceUpdateHandler := priceUpdateHandlerWrapper(*client.Database("database"))

	http.HandleFunc("/price_update", priceUpdateHandler)

	fmt.Println("Server listening on http://localhost:8080")
	http.ListenAndServe(":8080", nil)
}
