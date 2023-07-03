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
type SimpleEnumTypeSwitchVStr struct {
	Value string
}
type SimpleEnumTypeSwitchVTuple struct {
	A string
	B int64
}
type SimpleEnumTypeSwitchVStruct struct {
	Vfield string
}

func (SimpleEnumTypeSwitchVUnit) isSimpleEnumType()   {}
func (SimpleEnumTypeSwitchVTuple) isSimpleEnumType()  {}
func (SimpleEnumTypeSwitchVStr) isSimpleEnumType()    {}
func (SimpleEnumTypeSwitchVStruct) isSimpleEnumType() {}

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
	vtuple func(SimpleEnumTypeSwitchVTuple) (R, error),
	vstruct func(SimpleEnumTypeSwitchVStruct) (R, error),
) (res R, err error) {
	switch fieldType.(type) {
	case SimpleEnumTypeSwitchVUnit:
		if vunit != nil {
			return vunit(fieldType.(SimpleEnumTypeSwitchVUnit))
		}

	case SimpleEnumTypeSwitchVStr:
		if vstr != nil {
			return vstr(fieldType.(SimpleEnumTypeSwitchVStr))
		}

	case SimpleEnumTypeSwitchVTuple:
		if vtuple != nil {
			return vtuple(fieldType.(SimpleEnumTypeSwitchVTuple))
		}

	case SimpleEnumTypeSwitchVStruct:
		if vstruct != nil {
			return vstruct(fieldType.(SimpleEnumTypeSwitchVStruct))
		}

	default:
		return res, fmt.Errorf(
			"unsupported custom field type: '%s'", fieldType)
	}

	// If we get here, it's because we provided a nil function for a
	// type of custom field, implying we don't want to handle it.
	return res, nil
}

func Test() {
	var field SimpleEnumTypeSwitchVStr
	name, err := SimpleEnumTypeSwitch(field,
		func(setsv SimpleEnumTypeSwitchVUnit) (string, error) {
			return "VUnit", nil
		},
		func(setsv SimpleEnumTypeSwitchVStr) (string, error) {
			return "VStr", nil
		},
		func(setsv SimpleEnumTypeSwitchVTuple) (string, error) {
			return "VTuple", nil
		},
		func(setsv SimpleEnumTypeSwitchVStruct) (string, error) {
			return "VStruct", nil
		},
	)

	println("%s, %w", name, err)
}
