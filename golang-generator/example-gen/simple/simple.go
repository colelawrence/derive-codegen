package simple

import (
	"encoding/json"
	"fmt"
)

// `#[codegen(package = "simple", tags = "simple-go")]`
type SimpleEnum = SimpleEnumType

type SimpleEnumType interface{ isSimpleEnumType() }

// types
type SimpleEnum_VUnit struct{}
type SimpleEnum_VUnit2 struct{}
type SimpleEnum_VStr string
type SimpleEnum_VStr2 string
type SimpleEnum_VNewTypeStruct SimpleEnum_VTuple
type SimpleEnum_VTuple struct {
	A string
	B int64
}
type SimpleEnum_VStruct struct {
	Vfield map[string]uint
}

func (SimpleEnum_VUnit) isSimpleEnumType()          {}
func (SimpleEnum_VUnit2) isSimpleEnumType()         {}
func (SimpleEnum_VTuple) isSimpleEnumType()         {}
func (SimpleEnum_VStr) isSimpleEnumType()           {}
func (SimpleEnum_VStr2) isSimpleEnumType()          {}
func (SimpleEnum_VStruct) isSimpleEnumType()        {}
func (SimpleEnum_VNewTypeStruct) isSimpleEnumType() {}

func (v SimpleEnum_VUnit) MarshalJSON() ([]byte, error) {
	return json.Marshal("VUnit")
}

func (v *SimpleEnum_VUnit) UnmarshalJSON(b []byte) error {
	var a string

	if err := json.Unmarshal(b, &a); err != nil {
		return err
	}

	if a != "VUnit" {
		return fmt.Errorf("SimpleEnum::VUnit: bad value: %q", a)
	}

	*v = SimpleEnum_VUnit{}
	return nil
}

func MatchSimpleEnum[R any](
	fieldType SimpleEnumType,
	vunit func(SimpleEnum_VUnit) (R, error),
	vunit2 func(SimpleEnum_VUnit2) (R, error),
	vstr func(SimpleEnum_VStr) (R, error),
	vstr2 func(SimpleEnum_VStr2) (R, error),
	vtuple func(SimpleEnum_VTuple) (R, error),
	vnewtypestruct func(SimpleEnum_VNewTypeStruct) (R, error),
	vstruct func(SimpleEnum_VStruct) (R, error),
) (res R, err error) {
	switch v := fieldType.(type) {
	case SimpleEnum_VUnit:
		if vunit != nil {
			return vunit(v)
		}
	case SimpleEnum_VUnit2:
		if vunit != nil {
			return vunit2(v)
		}
	case SimpleEnum_VStr:
		if vstr != nil {
			return vstr(v)
		}
	case SimpleEnum_VStr2:
		if vstr != nil {
			return vstr2(v)
		}
	case SimpleEnum_VTuple:
		if vtuple != nil {
			return vtuple(v)
		}
	case SimpleEnum_VNewTypeStruct:
		if vtuple != nil {
			return vnewtypestruct(v)
		}
	case SimpleEnum_VStruct:
		if vstruct != nil {
			return vstruct(v)
		}
	default:
		return res, fmt.Errorf(
			"unsupported custom field type: '%s'", fieldType)
	}

	// If we get here, it's because we provided a nil function for a
	// type of custom field, implying we don't want to handle it.
	return res, nil
}

func getName(field SimpleEnumType) string {
	name, _ := MatchSimpleEnum(field,
		func(setsv SimpleEnum_VUnit) (string, error) {
			return "VUnit", nil
		},
		func(setsv SimpleEnum_VUnit2) (string, error) {
			return "VUnit2", nil
		},
		func(setsv SimpleEnum_VStr) (string, error) {
			return "VStr", nil
		},
		func(setsv SimpleEnum_VStr2) (string, error) {
			return "VStr2", nil
		},
		func(setsv SimpleEnum_VTuple) (string, error) {
			return "VTuple", nil
		},
		func(setsv SimpleEnum_VNewTypeStruct) (string, error) {
			return "VNewTypeStruct", nil
		},
		func(setsv SimpleEnum_VStruct) (string, error) {
			return "VStruct", nil
		},
	)

	// println("%s, %w", name, err)
	return name
}
