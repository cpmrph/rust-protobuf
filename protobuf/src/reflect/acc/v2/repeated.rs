use std::marker;

use crate::core::Message;

use crate::reflect::acc::v2::AccessorV2;
use crate::reflect::acc::FieldAccessor;
use crate::reflect::repeated::ReflectRepeated;
use crate::reflect::repeated::ReflectRepeatedMut;
use crate::reflect::repeated::ReflectRepeatedRef;
use crate::reflect::type_dynamic::ProtobufTypeDynamic;
use crate::reflect::types::ProtobufType;
use crate::reflect::ProtobufValueSized;

pub(crate) trait RepeatedFieldAccessor: Send + Sync + 'static {
    fn get_reflect<'a>(&self, m: &'a dyn Message) -> ReflectRepeatedRef<'a>;
    fn mut_reflect<'a>(&self, m: &'a mut dyn Message) -> ReflectRepeatedMut<'a>;
}

pub(crate) struct RepeatedFieldAccessorHolder {
    pub accessor: Box<dyn RepeatedFieldAccessor>,
    pub element_type: &'static dyn ProtobufTypeDynamic,
}

trait RepeatedFieldGetMut<M, R: ?Sized>: Send + Sync + 'static
where
    M: Message + 'static,
{
    fn get_field<'a>(&self, message: &'a M) -> &'a R;
    fn mut_field<'a>(&self, message: &'a mut M) -> &'a mut R;
}

struct RepeatedFieldGetMutImpl<M, L>
where
    M: Message + 'static,
{
    get_field: for<'a> fn(&'a M) -> &'a L,
    mut_field: for<'a> fn(&'a mut M) -> &'a mut L,
}

impl<M, V> RepeatedFieldGetMut<M, dyn ReflectRepeated> for RepeatedFieldGetMutImpl<M, Vec<V>>
where
    M: Message + 'static,
    V: ProtobufValueSized,
{
    fn get_field<'a>(&self, m: &'a M) -> &'a dyn ReflectRepeated {
        (self.get_field)(m) as &dyn ReflectRepeated
    }

    fn mut_field<'a>(&self, m: &'a mut M) -> &'a mut dyn ReflectRepeated {
        (self.mut_field)(m) as &mut dyn ReflectRepeated
    }
}

struct RepeatedFieldAccessorImpl<M, V>
where
    M: Message,
    V: ProtobufType,
{
    fns: Box<dyn RepeatedFieldGetMut<M, dyn ReflectRepeated>>,
    _marker: marker::PhantomData<V>,
}

impl<M, V> RepeatedFieldAccessor for RepeatedFieldAccessorImpl<M, V>
where
    M: Message,
    V: ProtobufType,
{
    fn get_reflect<'a>(&self, m: &'a dyn Message) -> ReflectRepeatedRef<'a> {
        let m = m.downcast_ref().unwrap();
        let repeated = self.fns.get_field(m);
        ReflectRepeatedRef { repeated }
    }

    fn mut_reflect<'a>(&self, m: &'a mut dyn Message) -> ReflectRepeatedMut<'a> {
        let m = m.downcast_mut().unwrap();
        let repeated = self.fns.mut_field(m);
        ReflectRepeatedMut { repeated }
    }
}

/// Make accessor for `Vec` field
pub fn make_vec_accessor<M, V>(
    name: &'static str,
    get_vec: for<'a> fn(&'a M) -> &'a Vec<V::ProtobufValue>,
    mut_vec: for<'a> fn(&'a mut M) -> &'a mut Vec<V::ProtobufValue>,
) -> FieldAccessor
where
    M: Message + 'static,
    V: ProtobufType + 'static,
{
    FieldAccessor::new_v2(
        name,
        AccessorV2::Repeated(RepeatedFieldAccessorHolder {
            accessor: Box::new(RepeatedFieldAccessorImpl::<M, V> {
                fns: Box::new(RepeatedFieldGetMutImpl::<M, Vec<V::ProtobufValue>> {
                    get_field: get_vec,
                    mut_field: mut_vec,
                }),
                _marker: marker::PhantomData::<V>,
            }),
            element_type: V::dynamic(),
        }),
    )
}
