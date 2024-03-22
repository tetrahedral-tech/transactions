package structs

import (
	"fmt"

	"go.mongodb.org/mongo-driver/bson/primitive"
)

type Status struct {
	Name string `bson:"name"`
	Time int    `bson:"time"`
}

type Account struct {
	ID        primitive.ObjectID `bson:"_id"`
	Algorithm primitive.ObjectID `bson:"algorithm"`
	Auth      string             `bson:"encryptedPrivateKey"` //@TODO update name
	Pair      Pair               `bson:"pair"`
	Provider  string             `bson:"provider"`
	Interval  int16              `bson:"interval"`
}

type Coin string
type Pair [2]Coin

func (pair Pair) String() string {
	return fmt.Sprintf("%s-%s", pair[0], pair[1])
}
