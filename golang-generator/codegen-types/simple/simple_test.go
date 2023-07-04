package simple

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestJson(t *testing.T) {
	val, err := json.Marshal("VUnit")
	require.Equal(t, nil, err)
	require.Equal(t, []byte("\"VUnit\""), val)
}

func TestJsonUnmarshal(t *testing.T) {
	input := []byte("   \"VUnit\"  ")
	var value interface{}
	err := json.Unmarshal(input, &value)
	require.Equal(t, nil, err)
	require.Equal(t, "VUnit", value)
}

func TestJsonUnmarshalNonstring(t *testing.T) {
	input := []byte("   [ \"VUnit\" ]  ")
	var value string
	err := json.Unmarshal(input, &value)
	require.NotNil(t, err)
}

func TestJsonUnmarshalVariant(t *testing.T) {
	input := []byte("   {\"VStr\": \"text\"}  ")
	var value struct {
		Data string `json:"VStr"`
	}
	err := json.Unmarshal(input, &value)
	require.Equal(t, nil, err)
}

func UnmarshalVariantsOfExternalTag(input string, into *SimpleEnum) (string, error) {
	inp := []byte(input)
	// external tagged unit types are represented as just strings
	var unitName string
	var err error
	err = json.Unmarshal(inp, &unitName)
	if err == nil {
		if unitName == "VUnit" {
			into.Data = SimpleEnumTypeSwitchVUnit{}
			return "VUnit", nil
		}
		return "", fmt.Errorf("Unmatched Variant %q", unitName)
	}

	var rawData map[string]json.RawMessage
	err = json.Unmarshal(inp, &rawData)
	if err != nil {
		return "", fmt.Errorf("Failed to parse JSON into raw map: %w", err)
	}
	for key, value := range rawData {
		switch key {
		case "VStr":
			var v SimpleEnumTypeSwitchVStr
			if err := json.Unmarshal(value, &v); err != nil {
				return "", fmt.Errorf("Error unmarshaling VStr variant: %w", err)
			}
			into.Data = v
			return "VStr", nil
			// var container struct {
			// 	Data string `json:"VStr"`
			// }
			// if err = json.Unmarshal(inp, &container); err == nil {
			// 	return "VStr", nil
			// }
		}
	}
	// Variants
	{
		var container struct {
			Data []interface{} `json:"VTuple"`
		}
		if err = json.Unmarshal(inp, &container); err == nil {
			raw := container.Data
			if len(raw) != 2 {
				return "", fmt.Errorf("Expected 2 items in `{\"VTuple\":[..]}`")
			}
			a, ok := raw[0].(string)
			if !ok {
				return "", fmt.Errorf("expected string for \"VTuple\" tuple's first element")
			}
			b, ok := raw[1].(float64)
			if !ok {
				return "", fmt.Errorf("expected number for \"VTuple\" tuple's second element")
			}
			into.Data = SimpleEnumTypeSwitchVTuple{
				A: a,
				B: int64(b),
			}
			return "VTuple", nil
		}
	}
	{
		var container struct {
			Data struct {
				A string
				B int64
			} `json:"VTuple"`
		}
		err = json.Unmarshal(inp, &container)
		if err == nil {
			return "VTuple", nil
		}
	}

	return "", fmt.Errorf("Unknown format")
}

func RequireJSONVariants(t *testing.T, output, input string) SimpleEnumType {
	t.Helper()
	var into SimpleEnum
	out, err := UnmarshalVariantsOfExternalTag(input, &into)
	require.Equal(t, nil, err)
	require.Equal(t, output, out)
	return into.Data
}

func TestJsonUnmarshalVariants(t *testing.T) {
	vunit := RequireJSONVariants(t, "VUnit", "\"VUnit\"")
	require.Equal(t, SimpleEnumTypeSwitchVUnit{}, vunit)
	vstr := RequireJSONVariants(t, "VStr", "   {\"VStr\": \"text\"}  ")
	require.Equal(t, SimpleEnumTypeSwitchVStr("text"), vstr)
	vtuple := RequireJSONVariants(t, "VTuple", "   {\"VTuple\": [\"text\",120]}  ")
	require.Equal(t, SimpleEnumTypeSwitchVTuple{A: "text", B: 120}, vtuple)
}

func TestMatchWithStr2(t *testing.T) {
	newTypeStr := SimpleEnumTypeSwitchVStr("Name")
	acceptsString(string(newTypeStr))
	require.Equal(t, "Hello Name", fmt.Sprintf("Hello %s", newTypeStr))
	require.Equal(t, "VStr", getName(SimpleEnumTypeSwitchVStr("SimpleEnumTypeSwitchVStr")))
	require.Equal(t, "VStr2", getName(SimpleEnumTypeSwitchVStr2("SimpleEnumTypeSwitchVStr")))
}

func acceptsString(str string) {}
