package structs

import (
	"regexp"

	"github.com/blackmesadev/black-mesa/consts"
)

type Untrustworthy struct {
	ID                string                   `json:"id,omitempty" bson:"id,omitempty"`
	Type              consts.UntrustworthyType `json:"type,omitempty" bson:"type,omitempty"`
	FilterString      string                   `json:"filterString,omitempty" bson:"filterString,omitempty"`
	FilterRegexString string                   `json:"filterRegexString,omitempty" bson:"filterRegexString,omitempty"`
	FilterRegexp      *regexp.Regexp           `json:"-" bson:"-"`
	Description       string                   `json:"description,omitempty" bson:"description,omitempty"`
	ReportedBy        string                   `json:"reportedBy,omitempty" bson:"reportedBy,omitempty"`
	ReportedOn        int64                    `json:"reportedOn,omitempty" bson:"reportedOn,omitempty"`
	ApprovedBy        string                   `json:"approvedBy,omitempty" bson:"approvedBy,omitempty"`
	ApprovedOn        int64                    `json:"approvedOn,omitempty" bson:"approvedOn,omitempty"`
	Approved          bool                     `json:"approved,omitempty" bson:"approved,omitempty"`
}
