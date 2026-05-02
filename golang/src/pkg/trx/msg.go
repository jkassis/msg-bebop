package trx

type Msg struct {
	ID     string   `json:"id"`
	FromID string   `json:"from_id"`
	ToIDs  []string `json:"to_ids"`
	Type   string   `json:"type_"`
	Version uint16  `json:"version,omitempty"`
	Body   []byte   `json:"body"`
}

func (m *Msg) Normalize() {
	if m.Version == 0 {
		m.Version = 1
	}
}

func NewMsg(id, fromID string, toIDs []string, type_ string, body []byte) Msg {
	return Msg{
		ID:      id,
		FromID:  fromID,
		ToIDs:   toIDs,
		Type:    type_,
		Version: 1,
		Body:    body,
	}
}
