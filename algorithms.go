package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"transactions/structs"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
)

type algorithm struct {
	id primitive.ObjectID `bson:"_id"`
	name string `bson:"name"`
}

func getAlgorithmIdToNameMap(database mongo.Database) (map[primitive.ObjectID]string, error) {
	idName := make(map[primitive.ObjectID]string)

	cursor, err := database.Collection("algorithms").Find(context.Background(), bson.M {})
	if err != nil {
		return nil, err
	}

	var results []algorithm
	cursor.All(context.Background(), &results)

	for _, algorithm := range results {
		idName[algorithm.id] = algorithm.name
	}

	return idName, nil
}

func getSignals(pair structs.Pair, interval int16) (map[string]structs.AlgorithmSignal, error) {
	url := fmt.Sprintf("http://127.0.0.1:5000/signals?pair=%s&interval=%d", pair.String(), interval)

	response, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	
	defer response.Body.Close()
	responseBody, err := io.ReadAll(response.Body)
	if err != nil {
		return nil, err
	}

	var parsedResponses map[string]structs.AlgorithmSignal
	err = json.Unmarshal(responseBody, &parsedResponses)
	if err != nil {
		return nil, err
	}

	return parsedResponses, nil
}