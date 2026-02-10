use proc_macro::{Ident, TokenStream};
use quote::{format_ident, quote};
use regex::Regex;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Lit, Meta, MetaNameValue};

#[proc_macro_derive(ToBasePlayer)]
pub fn to_base_player(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the name of the enum
    let enum_name = &input.ident;

    // Extract the data from the enum
    let enum_data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("ToBasePlayer can only be derived for enums."),
    };

    // Get the variants of the enum
    let variants = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #enum_name::#variant_name(p) => p,
        }
    });

    // Generate the implementation of ToBasePlayer
    let output = quote! {
        impl ToBasePlayer for #enum_name {
            fn get_player(&self) -> &dyn BasePlayer {
                match self {
                    #(#variants)*
                }
            }
        }
    };

    // Return the generated implementation as a TokenStream
    output.into()
}

#[proc_macro_derive(CustomDeserialize)]
pub fn deserialize_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let enum_data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("Can only apply to enum"),
    };

    let variants = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #enum_name::#variant_name(p) => p,
        }
    });

    // let variants = match &input.data {
    //     Data::Enum(data_enum) => &data_enum.variants,
    //     _ => panic!("Can only apply to enum"),
    // };

    // let arms = enum_data.variants.iter().map(|variant| {
    //     let ident = &variant.ident;
    //     quote! {
    //         Tagged::#ident(v) => Self::#ident(v),
    //     }
    // });

    let arms = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Deserialize into correct type for this variant
        let deser_stm = match &variant.fields {
            Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                let ty = &f.unnamed.first().unwrap().ty;
                quote!(let p: ::#ty = _)
            }
            _ => unimplemented!(), // TODO
        };

        quote! {
            #enum_name::#variant_name(#deser_stm) => p
        }
    });

    let expanded = quote! {
        impl<'de> Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(untagged)]
                enum Tagged {
                    #(#variants),*
                }

                let tagged = Tagged::deserialize(deserializer)?;
                Ok(match tagged {
                    #(#arms)*
                })
            }
        }
    };

    expanded.into()
}

// let expanded = quote! {
//     impl<'de> Deserialize<'de> for #enum_name {
//         fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//         where
//             D: Deserializer<'de>,
//         {
//             #[derive(Deserialize)]
//             #[serde(untagged)]
//             enum Tagged {
//                #(#variants),*
//             }

//             let tagged = Tagged::deserialize(deserializer)?;
//             Ok(match tagged {
//                 #(#arms)*
//             })
//         }
//     }
// };

#[proc_macro_derive(ImplBasePlayer)]
pub fn base_player_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Generate the implementation for Trait1
    let trait1_impl = generate_trait1_impl(&input);

    // Generate the implementation for Trait2
    let trait2_impl = generate_trait2_impl(&input);

    // Combine the generated code for both traits into a single TokenStream
    let generated_code = quote! {
        #trait1_impl
        #trait2_impl
    };

    // Return the generated implementation as a TokenStream
    generated_code.into()
}

fn generate_trait1_impl(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    // Extract the name of the struct
    let struct_name = &input.ident;

    let struct_base = parse_stat_value(struct_name.to_string().as_str());
    // let struct_core: Ident = Ident::new(struct_base.unwrap_or("XX".to_string()));
    let struct_core = struct_base.unwrap_or("XX".to_string());
    let generated_ident = format_ident!("{}", struct_core);

    // Generate the implementation of BasePlayer for the given struct
    quote! {
        impl BasePlayer for #struct_name {
            fn get_team(&self) -> TeamID {
                self.team.clone()
            }
            fn get_id(&self) -> String {
                self.id.clone()
            }
            fn get_name(&self) -> String {
                self.name.clone()
            }
            fn get_pos(&self) -> Position {
                return self.position;
            }
            fn get_json(&self) -> Value {
                let res = serde_json::to_value(self);
                match res {
                    Ok(js) => js,
                    Err(_) => json!(""),
                }
            }
            fn get_full_player(&self) -> Player {
                return Player::#generated_ident ((*self).clone());
            }
        }
    }
}

fn generate_trait2_impl(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &input.ident;
    quote! {
       impl ToBasePlayer for #struct_name {
            fn get_player(&self) -> &dyn BasePlayer {
                return self
            }
        }
    }
}

fn parse_stat_value(input: &str) -> Option<String> {
    // Define a regular expression to match either XX or X at the beginning of the string
    let re = Regex::new(r"^([A-Za-z]{1,2})Stats").unwrap();

    if let Some(captures) = re.captures(input) {
        // Extract the matched value from the first capture group
        let value = captures.get(1).unwrap().as_str().to_string();
        Some(value)
    } else {
        None
    }
}

#[proc_macro_derive(IsBlocker)]
pub fn is_blocker_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let name = input.ident;

    // Generate the code to implement the Blocker trait
    let expanded = quote! {
        impl Blocker for #name {
            fn get_blocks(&self) -> i32 {
                self.blocks
            }
        }
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(IsReceiver)]
pub fn is_receiver_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let name = input.ident;

    // Generate the code to implement the Blocker trait
    let expanded = quote! {
        impl Receiver for #name {
            fn get_pass_gain(&self) -> i32 {
                self.pass_gain()
            }
        }
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

// #[proc_macro_derive(IsBlocker)]
// pub fn is_blocker_derive(input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree
//     let input = parse_macro_input!(input as DeriveInput);

//     // Get the name of the struct
//     let name = input.ident;

//     // Generate the code to implement the Blocker trait
//     let expanded = quote! {
//         impl Blocker for #name {
//             fn get_blocks(&self) -> i32 {
//                 self.blocks
//             }
//         }
//     };

//     // Return the generated code as a TokenStream
//     TokenStream::from(expanded)
// }

// fn extract_player_info(attrs: &[Attribute]) -> String {
//     use syn::Meta;
//     use syn::parse::Parser;

//     // Define a parser to parse the 'player_info' attribute with a string value
//     let parser = syn::Attribute::parse_outer;

//     for attr in attrs {
//         if let Ok(Attribute { path, tokens, .. }) = parser.parse2(attr.tokens.clone()) {
//             if path.is_ident("player_info") {
//                 if let Ok(meta) = syn::parse2::<Meta>(tokens) {
//                     if let Meta::NameValue(nv) = meta {
//                         if nv.path.is_ident("value") {
//                             if let syn::Lit::Str(lit_str) = nv.lit {
//                                 return lit_str.value();
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     "".to_string() // Default value if the attribute is not provided
// }
// #[proc_macro_derive(ToBasePlayer)]
// pub fn to_base_player_derive(input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree
//     let ast = parse_macro_input!(input as DeriveInput);

//     // Extract the enum definition from the syntax tree
//     if let Data::Enum(data_enum) = ast.data {
//         // Collect match arms for each enum variant
//         let match_arms = data_enum.variants.iter().map(|variant| {
//             let ident = &variant.ident;
//             let variant_data = &variant.fields;

//             match variant_data {
//                 Fields::Named(_) => quote! {
//                     Self::#ident { ref p, .. } => p,
//                 },
//                 Fields::Unnamed(_) => quote! {
//                     Self::#ident(p) => p,
//                 },
//                 Fields::Unit => quote! {
//                     Self::#ident => panic!("Unexpected unit variant"),
//                 },
//             }
//         });

//         // Generate the implementation of ToBasePlayer trait
//         let gen = quote! {
//             impl ToBasePlayer for #ast {
//                 fn get_player(&self) -> &dyn BasePlayer {
//                     match self {
//                         #(#match_arms)*
//                     }
//                 }
//             }
//         };

//         // Return the generated tokens
//         gen.into()
//     } else {
//         // If the input is not an enum, return an error
//         panic!("ToBasePlayer can only be derived for enums");
//     }
// }
