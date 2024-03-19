package structs

import (
	"fmt"

	"go.mongodb.org/mongo-driver/bson/primitive"
)

type Coin struct {
	Name string
	supportedProviders []string
}

func (coin Coin) String() string {
	return coin.Name
}

type Pair struct {
	A Coin
	B Coin
}

func (pair Pair) String() string {
	return fmt.Sprintf("%s-%s", pair.A.String(), pair.B.String())
}

type Status struct {
	Name string `bson:"name"`
	Time int    `bson:"time"`
}

type Account struct {
	ID                  primitive.ObjectID `bson:"_id"`
	Algorithm           primitive.ObjectID `bson:"algorithm"`
	EncryptedPrivateKey string             `bson:"encryptedPrivateKey"`
	Pair                Pair               `bson:"pair"`
	Provider            string             `bson:"provider"`
	Interval            int16              `bson:"interval"`
}
