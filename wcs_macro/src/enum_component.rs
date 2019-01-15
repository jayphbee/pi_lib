use crate::proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;

use util::*;

pub fn impl_enum_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let p_name = point_name(name.clone().to_string());
    let g_name = group_name(name.clone().to_string());
    let r_r_name = read_ref_name(name.clone().to_string());
    let w_r_name = write_ref_name(name.clone().to_string());
    let gen = match &ast.data {
        syn::Data::Enum(e) => {
            let mut members: Vec<(syn::Ident, syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, bool)> = Vec::new();
            for v in e.variants.iter(){
                let i = v.ident.clone();
                let mut is_name = false;
                let fs = match &v.fields{
                    syn::Fields::Named(ref fs) => {is_name = true; fs.named.clone()},
                    syn::Fields::Unnamed(ref fs) => fs.unnamed.clone(),
                    syn::Fields::Unit => panic!("type error"),
                };
                members.push((i, fs, is_name));
            }
            let p = impl_point_named(&p_name, &members);
            let r_r = impl_ref_named(&r_r_name, false, &p_name, &g_name, &members);
            let w_r = impl_ref_named(&w_r_name, true, &p_name, &g_name, &members);
            let g = impl_group_named(&g_name, &members);

            quote!{
                #p
                #r_r
                #w_r
                #g
            }
        },
        _ => panic!("type error"),
    };
    gen.into()
}

fn impl_point_named(p_name: &syn::Ident, members: &Vec<(syn::Ident, syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, bool)>) -> quote::__rt::TokenStream {
    let mut point_impls = Vec::new();
    for member in members.iter(){
        let mut arr_point = Vec::new();
        let mut arr_name = Vec::new();
        let name = &member.0;
        for field in member.1.iter(){
            arr_point.push(point_name(field.ty.clone().into_token_stream().to_string()));
            arr_name.push(field.ident.clone());
        }
        if member.2{
            point_impls.push(quote!{
                #name{
                    #(#arr_name: #arr_point),*
                }
            });
        }else {
            point_impls.push(quote!{
                #name(#(#arr_point),*)
            });
        }
    }
    quote!{
        pub enum #p_name{
            #(#point_impls),*
        }
    }
}

fn impl_ref_named(r_name: &syn::Ident, is_write: bool, p_name: &syn::Ident, g_name: &syn::Ident, members: &Vec<(syn::Ident, syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, bool)>) -> quote::__rt::TokenStream {
    let mut ref_impls = Vec::new();
    let mut match_points = Vec::new();
    let mut g_clones = Vec::new();
    let mut pns = Vec::new();
    let mut rns = Vec::new();
    let mut i = 0;
    for member in members.iter(){
        let mut arr_ref = Vec::new();
        let mut arr_point = Vec::new();
        let mut arr_name = Vec::new();
        let mut arr_i = Vec::new();
        let mut arr_index = Vec::new();
        let mut arr_index_str = Vec::new();
        let name = &member.0;
        pns.push(p_name.clone());
        rns.push(r_name.clone());
        let mut j = 0;
        for field in member.1.iter(){
            arr_point.push(point_name(field.ty.clone().into_token_stream().to_string()));
            if is_write {
                arr_ref.push(write_ref_name(field.ty.clone().into_token_stream().to_string()));
            }else {
                arr_ref.push(read_ref_name(field.ty.clone().into_token_stream().to_string()));
            }
            
            if member.2 {
                arr_name.push(field.ident.clone());
            }else {
                arr_index.push(j);
                arr_index_str.push(ident(&("_".to_string() + j.to_string().as_str())));
                j += 1;
            }
            
            arr_i.push(i);
        }

        if member.2 {
            let mut arr_name1 = arr_name.clone();
            let mut arr_name2 = arr_name.clone();
            let mut arr_name3 = arr_name.clone();
            let mut arr_name4 = arr_name.clone();
            let mut arr_ref1 = arr_ref.clone();
            ref_impls.push(quote!{
                #name{
                    #(#arr_name: #arr_ref<M>),*
                }
            });
            match_points.push(quote! {
                #name{#(#arr_name1),*}
            });
            if is_write {
                g_clones.push(quote!{
                    #name{#(#arr_name2: #arr_ref1::new(#arr_name4, g.#arr_i.#arr_name3.to_usize(), m) ),*}
                });
            }else {
                g_clones.push(quote!{
                    #name{#(#arr_name2: #arr_ref1::new(#arr_name4, &g.#arr_i.#arr_name3) ),*}
                });
            }
            
        }else {
            let mut arr_index_str1 = arr_index_str.clone();
            let mut arr_index_str2 = arr_index_str.clone();
            let mut arr_ref1 = arr_ref.clone();
            ref_impls.push(quote!{
                #name(#(#arr_ref<'a, M>),*)
            });
            match_points.push(quote! {
                #name(#(#arr_index_str1),*)
            });
            if is_write {
                g_clones.push(quote!{
                    #name(#(#arr_ref1::new(#arr_index_str2, (g.#arr_i).#arr_index.to_usize(), m) ),*)
                });
            }else {
                g_clones.push(quote!{
                    #name(#(#arr_ref1::new(#arr_index_str2, &(g.#arr_i).#arr_index) ),*)
                });
            }
        }
        i += 1;
        
    }
    if is_write {
        quote!{
            pub enum #r_name<'a, M: ComponentMgr>{
                #(#ref_impls),*
            }

            impl<'a, M: ComponentMgr> #r_name<'a, M>{
                pub fn new(p: #p_name, g: usize, m: &mut M) -> #r_name<M>{
                    let g = #g_name::<M>::from_usize_mut(g);
                    match p {
                        #(#pns::#match_points => #rns::#g_clones),*
                    }
                }
            }
        }
    }else {
        quote!{
            pub enum #r_name<'a, M: ComponentMgr>{
                #(#ref_impls),*
            }

            impl<'a, M: ComponentMgr> #r_name<'a, M>{
                pub fn new(p: #p_name, g: &#g_name<M>) -> #r_name<M>{
                    match p {
                        #(#pns::#match_points => #rns::#g_clones),*
                    }
                }
            }
        }
    }
    
}

fn impl_group_named(g_name: &syn::Ident, members: &Vec<(syn::Ident, syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, bool)>) -> quote::__rt::TokenStream {
    let mut member_impls = Vec::new();
    let mut new_impls = Vec::new();
    // let mut set_mgr_impls = Vec::new();
    let mut i = 0;
    let mut arr_names = Vec::new();
    for member in members.iter(){
        let mut arr_group = Vec::new();
        let mut arr_name = Vec::new();
        let mut arr_index = Vec::new();
        let mut arr_i = Vec::new();
        
        let name = &member.0;
        let mut j = 0;
        for field in member.1.iter(){
            arr_group.push(group_name(field.ty.clone().into_token_stream().to_string()));
            if member.2{
                arr_name.push(field.ident.clone());
            }else {
                arr_index.push(j);
                j += 1;
            }
            arr_name.push(field.ident.clone());
            arr_i.push(i);
        }

        if member.2{
            let mut arr_name1 = arr_name.clone();
            // let mut arr_name2 = arr_name.clone();
            let mut arr_group1 = arr_group.clone();
            let member_name = ident(&(name.clone().to_string() + g_name.to_string().as_str()));
            arr_names.push(member_name.clone());
            member_impls.push(quote!{
                pub struct #member_name<M: ComponentMgr>{
                    #(#arr_name: #arr_group<M>),*
                }
            });
            new_impls.push(quote! {
                #member_name{#(#arr_name1: #arr_group1::new()),*}
            });
            // set_mgr_impls.push(quote!{
            //     #(self.#arr_i.#arr_name2.borrow_mut().set_mgr(mgr.clone()));*
            // });
        }else {
            let mut arr_group1 = arr_group.clone();
            let member_name = ident(&(name.clone().to_string() + g_name.to_string().as_str()));
            arr_names.push(member_name.clone());
            member_impls.push(quote!{
                pub struct #member_name<M: ComponentMgr>(#(#arr_group<M>),*);
            });
            new_impls.push(quote! {
                #member_name(#(#arr_group1::new()),*)
            });
            // set_mgr_impls.push(quote!{
            //     #((self.#arr_i).#arr_index.borrow_mut().set_mgr(mgr.clone()));*
            // });
        }
        
        i += 1;
    }
    
    quote!{
        #(#member_impls)*

        pub struct #g_name<M: ComponentMgr>(#(#arr_names<M>),*);

        impl<M: ComponentMgr> ComponentGroupTree for #g_name<M>{
            type C = M;
            fn new () -> #g_name<M>{
                #g_name(#(#new_impls),*)
            }

            // fn set_mgr(&mut self, mgr: Weak<RefCell<Self::C>>){
            //     #(#set_mgr_impls);*
            // }
        }

        impl<M: ComponentMgr> #g_name<M>{
            #[inline]
            pub fn to_usize (&self) -> usize{
                self as *const #g_name<M> as usize
            }

            #[inline]
            pub fn from_usize<'a> (ptr: usize) -> &'a #g_name<M>{
                unsafe{&*(ptr as *const #g_name<M>)}
            }

            #[inline]
            pub fn from_usize_mut<'a>(ptr: usize) -> &'a mut #g_name<M>{
                unsafe{&mut *(ptr as *mut #g_name<M>)}
            }
        }
    }
}