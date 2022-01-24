package structs

type BlackMesaUser struct {
	ID      string `json:"id" bson:"id"`
	Trusted bool   `json:"trusted" bson:"trusted"`
	Owner   bool   `json:"owner" bson:"owner"`
}
