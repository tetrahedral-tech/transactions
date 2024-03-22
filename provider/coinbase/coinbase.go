package coinbase

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"time"
	"transactions/structs"
)

type Provider struct {
	BaseUrl string
	key     string
}

type verification struct{}
type auth struct {
	Key        string
	Signature  string
	Timestamp  int64
	Passphrase string
}
type signatureRouteData struct {
	Method      string
	RequestPath string
	Body        map[string]string
}

func (provider Provider) generateSignature(routeData signatureRouteData) (*auth, error) {
	timestamp := time.Now().Unix()

	key, err := base64.StdEncoding.DecodeString(provider.key)
	if err != nil {
		return nil, err
	}

	marshalledBody, err := json.Marshal(routeData.Body)
	if err != nil {
		return nil, err
	}

	message := []byte(fmt.Sprintf("%d%s%s%s", timestamp, routeData.Method, routeData.RequestPath, marshalledBody))

	hash := hmac.New(sha256.New, key)
	hash.Write(message)

	return &auth{
		Key:        provider.key,
		Timestamp:  timestamp,
		Passphrase: "", //@TODO maybe?
		Signature:  base64.StdEncoding.EncodeToString(hash.Sum(nil)),
	}, nil
}

func (provider Provider) Swap(account structs.Account, transaction structs.TransactionInfo) (*structs.TransactionResult, error) {
	// @TODO
	err := provider.Verify(nil)
	if err != nil {
		return nil, err
	}

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

	return fmt.Errorf("disabled temporarily")
	return nil
}

func verifySecret(key string) (bool, error) {
	// @TODO
	return true, nil
}

func NewProvider(key string) (*Provider, error) {
	provider := Provider{
		BaseUrl: "https://api-public.sandbox.exchange.coinbase.com",
		key:     key,
	}

	valid, err := verifySecret(key)
	if err != nil {
		return nil, err
	}

	if !valid {
		return nil, fmt.Errorf("invalid auth data")
	}

	return &provider, nil
}
