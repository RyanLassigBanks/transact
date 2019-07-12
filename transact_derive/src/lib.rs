// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![recursion_limit = "128"]
extern crate proc_macro;

mod builder;
mod protos;

use builder::generate_builder_macro;
use proc_macro::TokenStream;
use protos::{
    generate_from_bytes, generate_from_native, generate_from_proto, generate_into_bytes,
    generate_into_native, generate_into_proto,
};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

///! Generates builder struct and getter methods
///
/// Attributes
///
/// `#[builder_name]`
///
/// Gives builder struct a specific name. If this attribute is not included,
/// the builders name will be derived from the struct
///
/// #[derive(Builder)]
/// #[gen_build_impl]
/// #[builder_name = "Bar"]
/// pub struct Foo {
///   #[getter]
///   name: String
/// }
///
/// let foo = Bar::new().with_name("foo");
///
/// `#[gen_build_impl]`
///
/// When included in struct definition, `Builder` will generate a default `build`
/// method for builder. If `gen_build_impl` is not included the user will need to
/// implement to `build` trait manually.
///
/// `#[getter]`
///
/// When applied to a struct field, a corresponding get method will be generated for
/// field. generated getters return references to field value.
///
/// #[derive(Builder)]
/// #[gen_build_impl]
/// pub struct Foo {
///   #[getter]
///   bar: String
/// }
///
/// let foo = FooBuilder::new()
///    .with_bar("cool bar")
///    .build()
///    .unwrap();
///
/// assert_eq!("cool bar", foo.bar());
///
/// `#[optional]`
///
/// Struct fields that have the `optional` are not required to be set in order to build
/// the struct using the default build method.
///
/// #[derive(Builder)]
/// #[gen_build_impl]
/// pub struct Foo {
///   #[getter]
///   bar: String,
///
///   #[optional]
///   baz: u32
/// }
///
/// let foo = FooBuilder::new()
///    .with_bar("cool bar")
///    .build();
///
/// assert!(foo.is_ok());
///
/// let foo2 = FooBuilder::new()
///    .with_baz(23)
///    .build();
///
/// assert!(foo2.is_err());
#[proc_macro_derive(Builder, attributes(builder_name, gen_build_impl, getter, optional))]
pub fn derive_builder(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    generate_builder_macro(derive_input)
        .map(|t| t.into())
        .unwrap_or_else(|err| {
            let compile_error = err.to_compile_error();
            quote!(#compile_error).into()
        })
}

/// Generates an implementation of the `from_proto` trait
///
/// Attributes
///
/// `#[proto_type]`
///
/// Required. Used to specify to path to protobuf implementation `from_proto` will
/// take as an argument.
///
/// #[derive(FromProtoImpl)]
/// #[proto_type = "protos::processor::TpRegisterRequest"]
/// pub struct Foo {
/// }
///
/// `#[proto_enum]`
///
/// Used by enums to map the native enum to the protobuf enum.
///
/// Sample Usage:
///
/// ProtoBuf enum `TpProcessRequestHeaderStyle`
///
/// message TpRegisterRequest {
///    enum TpProcessRequestHeaderStyle {
///        HEADER_STYLE_UNSET = 0;
///        EXPANDED = 1;
///        RAW = 2;
///     }
/// }
///
/// Corresponding native implementation
///
/// #[derive(FromProtoImpl)]
/// #[proto_type = "protos::processor::TpRegisterRequest_TpProcessRequestHeaderStyle"]
/// pub enum TpProcessRequestHeaderStyle {
///    #[proto_enum(HEADER_STYLE_UNSET)]
///     HeaderStyleUnset,
///
///    #[proto_enum(EXPANDED)]
///    Expanded,
///
///    #[proto_enum(RAW)]
///    Raw,
/// }
///
/// `#[from_proto_impl]`
///
/// When added to a field, this attribute provides direction on how to convert the corresponding
/// protobuf field into a native field.
///
/// Accepted directives:
///   * to_string
///   * clone
///   * Vec
///   * from_proto
///
/// #[from_impl(to_string)]
/// foo: String
///
/// Generates
///
/// foo: proto.get_foo().to_string()
///
/// #[from_impl(clone)]
/// foo: String
///
/// Generates
///
/// foo: proto.get_foo().clone()
///
/// #[from_impl(from_proto)]
/// foo: Foo
///
/// Generates
///
/// foo: Foo::from_proto(proto.get_foo().clone())?
///
/// #[from_impl(Vec)]
/// foo: Vec<Foo>
///
/// Generates
///
/// foo: proto.get_foo().to_vec().into_iter().map(Foo::from_proto).collect()
///
#[proc_macro_derive(FromProtoImpl, attributes(proto_type, from_proto_impl, proto_enum))]
pub fn derive_from_proto(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    generate_from_proto(derive_input)
        .map(|t| t.into())
        .unwrap_or_else(|err| {
            let compile_error = err.to_compile_error();
            quote!(#compile_error).into()
        })
}

/// Generates an implementation of the `from_native` trait
///
/// Attributes
///
/// `#[proto_type]`
///
/// Required. Used to specify to path to protobuf implementation `from_native` will
/// return.
///
/// #[derive(FromProtoImpl)]
/// #[proto_type = "protos::processor::TpRegisterRequest"]
/// pub struct Foo {
/// }
///
/// `#[proto_enum]`
///
/// Used by enums to map the native enum to the protobuf enum.
///
/// Sample Usage:
///
/// ProtoBuf enum `TpProcessRequestHeaderStyle`
///
/// message TpRegisterRequest {
///    enum TpProcessRequestHeaderStyle {
///        HEADER_STYLE_UNSET = 0;
///        EXPANDED = 1;
///        RAW = 2;
///     }
/// }
///
/// Corresponding native implementation
///
/// #[derive(FromProtoImpl)]
/// #[proto_type = "protos::processor::TpRegisterRequest_TpProcessRequestHeaderStyle"]
/// pub enum TpProcessRequestHeaderStyle {
///    #[proto_enum(HEADER_STYLE_UNSET)]
///     HeaderStyleUnset,
///
///    #[proto_enum(EXPANDED)]
///    Expanded,
///
///    #[proto_enum(RAW)]
///    Raw,
/// }
///
/// `#[native_proto_impl]`
///
/// When added to a field, this attribute provides direction on how to convert the corresponding
/// native field into a protobuf field.
///
/// Accepted directives:
///   * to_string
///   * clone
///   * into_proto
///   * Vec
///   * deref
///
/// #[from_impl(to_string)]
/// foo: String
///
/// Generates
///
/// proto.set_foo(native.foo().to_string());
///
/// #[from_impl(clone)]
/// foo: String
///
/// Generates
///
/// proto.set_foo(native.foo().clone());
///
/// #[from_impl(into_proto)]
/// foo: Foo
///
/// Generates
///
/// proto.set_foo(native.foo().into_proto()?);
///
/// #[from_impl(Vec)]
/// foo: Vec<Foo>
///
/// Generates
///
/// proto.set_foo(RepeatedField::from_vec(native.foo().to_vec().into_iter().map(Foo::into_proto).collect()));
///
#[proc_macro_derive(FromNativeImpl, attributes(proto_type, from_native_impl, proto_enum))]
pub fn derive_from_native(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    generate_from_native(derive_input)
        .map(|t| t.into())
        .unwrap_or_else(|err| {
            let compile_error = err.to_compile_error();
            quote!(#compile_error).into()
        })
}

/// Generates `into_proto` implementation for struct
#[proc_macro_derive(IntoProtoImpl)]
pub fn derive_into_proto(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    generate_into_proto(derive_input)
        .map(|t| t.into())
        .unwrap_or_else(|err| {
            let compile_error = err.to_compile_error();
            quote!(#compile_error).into()
        })
}

/// Generates `into_native` implementation for struct
#[proc_macro_derive(IntoNativeImpl)]
pub fn derive_into_native(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    generate_into_native(derive_input)
        .map(|t| t.into())
        .unwrap_or_else(|err| {
            let compile_error = err.to_compile_error();
            quote!(#compile_error).into()
        })
}

/// Generates `into_bytes` implementation for struct
#[proc_macro_derive(IntoBytesImpl)]
pub fn derive_into_bytes(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    generate_into_bytes(derive_input)
        .map(|t| t.into())
        .unwrap_or_else(|err| {
            let compile_error = err.to_compile_error();
            quote!(#compile_error).into()
        })
}

/// Generates `into_bytes` implementation for struct
///
/// Attributes
///
/// `#[proto_type]`
///
/// Required. Used to specify to path to protobuf implementation `from_native` will
/// return.
///
/// #[derive(FromProtoImpl)]
/// #[proto_type = "protos::processor::TpRegisterRequest"]
/// pub struct Foo {
/// }
#[proc_macro_derive(FromBytesImpl, attributes(proto_type))]
pub fn derive_from_bytes(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    generate_from_bytes(derive_input)
        .map(|t| t.into())
        .unwrap_or_else(|err| {
            let compile_error = err.to_compile_error();
            quote!(#compile_error).into()
        })
}
