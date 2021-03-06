package discord

import (
	"strings"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

// Route holds information about a specific message route handler
type Route struct {
	Pattern     string      // match pattern that should trigger this route handler
	Description string      // short description of this route
	Help        string      // detailed help string for this route
	Run         HandlerFunc // route handler function to call
}

// HandlerFunc is the function signature required for a message route handler.
type HandlerFunc func(*discordgo.Session, *structs.Config, *discordgo.Message, *discordgo.Context, []string)

// Mux is the main struct for all mux methods.
type Mux struct {
	Routes  []*Route
	Default *Route
}

// New returns a new Discord message route mux
func NewRouter() *Mux {
	m := &Mux{}
	m.Routes = make([]*Route, 0, 0)
	return m
}

// Route allows you to register a route
func (m *Mux) Route(pattern string, handler HandlerFunc) (*Route, error) {

	r := Route{}
	r.Pattern = pattern
	r.Run = handler
	m.Routes = append(m.Routes, &r)

	return &r, nil
}

func (m *Mux) Match(msg string) (*Route, []string, []string) {

	fields := strings.Fields(msg)

	if len(fields) == 0 {
		return nil, nil, nil
	}
	var r *Route

	fieldCount := len(fields)
	field := fields[0]
	for _, route := range m.Routes {
		if route.Pattern == field {
			return route, fields[fieldCount:], fields[1:]
		}
	}
	return r, fields[fieldCount:], fields[1:]
}
