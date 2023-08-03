package simple

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestJson(t *testing.T) {
	val, err := json.Marshal("VUnit")
	require.NoError(t, err)
	require.Equal(t, []byte("\"VUnit\""), val)
}

func TestJsonUnmarshal(t *testing.T) {
	input := []byte("   \"VUnit\"  ")
	var value interface{}
	err := json.Unmarshal(input, &value)
	require.NoError(t, err)
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
	require.NoError(t, err)
}

func (dest *SimpleEnum_VTuple) UnmarshalJSON(data []byte) error {
	var v []interface{}
	if err := json.Unmarshal(data, &v); err != nil {
		return fmt.Errorf("Error unmarshaling VTuple variant as array: %w", err)
	}

	if len(v) != 2 {
		return fmt.Errorf("Expected 2 items in `{\"VTuple\":[..]}`")
	}
	a, ok := v[0].(string)
	if !ok {
		return fmt.Errorf("expected string for \"VTuple\" tuple's first element")
	}
	b, ok := v[1].(float64)
	if !ok {
		return fmt.Errorf("expected number for \"VTuple\" tuple's second element")
	}
	*dest = SimpleEnum_VTuple{
		A: a,
		B: int64(b),
	}
	return nil
}

func (dest *SimpleEnum_VNewTypeStruct) UnmarshalJSON(data []byte) error {
	var v SimpleEnum_VTuple
	if err := json.Unmarshal(data, &v); err != nil {
		return fmt.Errorf("Error unmarshaling VNewTypeStruct variant as SimpleEnum_VTuple: %w", err)
	}
	*dest = SimpleEnum_VNewTypeStruct(v)
	return nil
}

func UnmarshalVariantsOfExternalTag(input string, into *SimpleEnum) (string, error) {
	inp := []byte(input)
	// external tagged unit types are represented as just strings
	var unitName string
	var err error
	if err = json.Unmarshal(inp, &unitName); err == nil {
		switch unitName {
		case "VUnit":
			*into = SimpleEnum(SimpleEnum_VUnit{})
			return "VUnit", nil
		case "VUnit2":
			*into = SimpleEnum(SimpleEnum_VUnit2{})
			return "VUnit2", nil
		default:
			return "", fmt.Errorf("Unmatched Variant %q", unitName)
		}
	}

	var rawData map[string]json.RawMessage
	err = json.Unmarshal(inp, &rawData)
	if err != nil {
		return "", fmt.Errorf("Failed to parse JSON into raw map: %w", err)
	}
	for key, value := range rawData {
		switch key {
		case "VStr":
			var v SimpleEnum_VStr
			if err := json.Unmarshal(value, &v); err != nil {
				return "", fmt.Errorf("Error unmarshaling VStr variant: %w", err)
			}
			*into = SimpleEnum(v)
			return "VStr", nil
		case "VStr2":
			var v SimpleEnum_VStr2
			if err := json.Unmarshal(value, &v); err != nil {
				return "", fmt.Errorf("Error unmarshaling VStr2 variant: %w", err)
			}
			*into = SimpleEnum(v)
			return "VStr2", nil
		case "VTuple":
			var v SimpleEnum_VTuple
			if err := json.Unmarshal(value, &v); err != nil {
				return "", fmt.Errorf("Error unmarshaling VTuple variant: %w", err)
			}
			*into = SimpleEnum(v)
			return "VTuple", nil
		case "VNewTypeStruct":
			var v SimpleEnum_VNewTypeStruct
			if err := json.Unmarshal(value, &v); err != nil {
				return "", fmt.Errorf("Error unmarshaling VNewTypeStruct variant: %w", err)
			}
			*into = SimpleEnum(v)
			return "VNewTypeStruct", nil
		case "VStruct":
			var v SimpleEnum_VStruct
			if err := json.Unmarshal(value, &v); err != nil {
				return "", fmt.Errorf("Error unmarshaling VStruct variant: %w", err)
			}
			*into = SimpleEnum(v)
			return "VStruct", nil
		}
	}

	return "", fmt.Errorf("Unknown SimpleEnum variant from %v", input)
}

func RequireJSONVariants(t *testing.T, output, input string) SimpleEnumType {
	t.Helper()
	var into SimpleEnum
	out, err := UnmarshalVariantsOfExternalTag(input, &into)
	require.NoError(t, err)
	require.Equal(t, output, out)
	return into
}

func TestJsonUnmarshalVariants(t *testing.T) {
	vunit := RequireJSONVariants(t, "VUnit", "\"VUnit\"")
	require.Equal(t, SimpleEnum_VUnit{}, vunit)
	vunit2 := RequireJSONVariants(t, "VUnit2", "  \"VUnit2\" ")
	require.Equal(t, SimpleEnum_VUnit2{}, vunit2)
	vstr := RequireJSONVariants(t, "VStr", "   {\"VStr\": \"text 1\"}  ")
	require.Equal(t, SimpleEnum_VStr("text 1"), vstr)
	vstr2 := RequireJSONVariants(t, "VStr2", "{\"VStr2\": \"text 2\"}")
	require.Equal(t, SimpleEnum_VStr2("text 2"), vstr2)
	vtuple := RequireJSONVariants(t, "VTuple", "   {\"VTuple\": [\"text\",120]}  ")
	require.Equal(t, SimpleEnum_VTuple{A: "text", B: 120}, vtuple)
	vnewtypestruct := RequireJSONVariants(t, "VNewTypeStruct", "   {\"VNewTypeStruct\": [\"string\", 120]}  ")
	require.Equal(t, SimpleEnum_VNewTypeStruct(SimpleEnum_VTuple{A: "string", B: 120}), vnewtypestruct)
	vstruct := RequireJSONVariants(t, "VStruct", "   {\"VStruct\": {\"vfield\": \"...\"}}  ")
	require.Equal(t, SimpleEnum_VStruct{Vfield: "..."}, vstruct)
}

func TestMatchWithStr2(t *testing.T) {
	newTypeStr := SimpleEnum_VStr("Name")
	acceptsString(string(newTypeStr))
	require.Equal(t, "Hello Name", fmt.Sprintf("Hello %s", newTypeStr))
	require.Equal(t, "VStr", getName(SimpleEnum_VStr("SimpleEnum_VStr")))
	require.Equal(t, "VStr2", getName(SimpleEnum_VStr2("SimpleEnum_VStr")))
}

func acceptsString(str string) {}
