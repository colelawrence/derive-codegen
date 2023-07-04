package simple

import (
	"encoding/json"
	"fmt"
)

type SimpleEnum struct {
	Data SimpleEnumType
}

type SimpleEnumType interface{ isSimpleEnumType() }

type SimpleEnumTypeSwitchVUnit struct{}
type SimpleEnumTypeSwitchVStr string
type SimpleEnumTypeSwitchVStr2 string
type SimpleEnumTypeSwitchVNewTypeStruct SimpleEnumTypeSwitchVTuple
type SimpleEnumTypeSwitchVTuple struct {
	A string
	B int64
}
type SimpleEnumTypeSwitchVStruct struct {
	Vfield string
}

func (SimpleEnumTypeSwitchVUnit) isSimpleEnumType()          {}
func (SimpleEnumTypeSwitchVTuple) isSimpleEnumType()         {}
func (SimpleEnumTypeSwitchVStr) isSimpleEnumType()           {}
func (SimpleEnumTypeSwitchVStr2) isSimpleEnumType()          {}
func (SimpleEnumTypeSwitchVStruct) isSimpleEnumType()        {}
func (SimpleEnumTypeSwitchVNewTypeStruct) isSimpleEnumType() {}

func (v SimpleEnumTypeSwitchVUnit) MarshalJSON() ([]byte, error) {
	return json.Marshal("VUnit")
}

func (v *SimpleEnumTypeSwitchVUnit) UnmarshalJSON(b []byte) error {
	var a string

	if err := json.Unmarshal(b, &a); err != nil {
		return err
	}

	if a != "VUnit" {
		return fmt.Errorf("SimpleEnum::VUnit: bad value: %q", a)
	}

	*v = SimpleEnumTypeSwitchVUnit{}
	return nil
}

func SimpleEnumTypeSwitch[R any](
	fieldType SimpleEnumType,
	vunit func(SimpleEnumTypeSwitchVUnit) (R, error),
	vstr func(SimpleEnumTypeSwitchVStr) (R, error),
	vstr2 func(SimpleEnumTypeSwitchVStr2) (R, error),
	vtuple func(SimpleEnumTypeSwitchVTuple) (R, error),
	vstruct func(SimpleEnumTypeSwitchVStruct) (R, error),
) (res R, err error) {
	switch v := fieldType.(type) {
	case SimpleEnumTypeSwitchVUnit:
		if vunit != nil {
			return vunit(v)
		}
	case SimpleEnumTypeSwitchVStr:
		if vstr != nil {
			return vstr(v)
		}
	case SimpleEnumTypeSwitchVStr2:
		if vstr != nil {
			return vstr2(v)
		}
	case SimpleEnumTypeSwitchVTuple:
		if vtuple != nil {
			return vtuple(v)
		}
	case SimpleEnumTypeSwitchVStruct:
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
	name, _ := SimpleEnumTypeSwitch(field,
		func(setsv SimpleEnumTypeSwitchVUnit) (string, error) {
			return "VUnit", nil
		},
		func(setsv SimpleEnumTypeSwitchVStr) (string, error) {
			return "VStr", nil
		},
		func(setsv SimpleEnumTypeSwitchVStr2) (string, error) {
			return "VStr2", nil
		},
		func(setsv SimpleEnumTypeSwitchVTuple) (string, error) {
			return "VTuple", nil
		},
		func(setsv SimpleEnumTypeSwitchVStruct) (string, error) {
			return "VStruct", nil
		},
	)

	// println("%s, %w", name, err)
	return name
}
