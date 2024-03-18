package provider

type TradeProvider interface {
	Swap()
	Verify()
}
