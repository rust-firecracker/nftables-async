use std::future::Future;

use nftables::{
    batch::Batch,
    helper::NftablesError,
    schema::{NfListObject, NfObject, Nftables, Table},
    types::NfFamily,
};
use nftables_async::{
    apply_ruleset, get_current_ruleset,
    process::{AsyncProcess, TokioProcess},
};

#[test]
fn can_apply_ruleset() {
    run_test(|is_tokio| async move {
        let mut batch = Batch::new();
        batch.add_obj(NfListObject::Table(Table {
            family: NfFamily::INet,
            name: format!("table{}", fastrand::u32(1..=10000)).into(),
            handle: None,
        }));

        wrapped_apply_ruleset(batch.to_nftables(), is_tokio)
            .await
            .unwrap();
    });
}

#[test]
fn can_get_current_ruleset() {
    run_test(|is_tokio| async move {
        let mut batch = Batch::new();
        let table_name = format!("table{}", fastrand::u32(1..=10000));
        batch.add_obj(NfListObject::Table(Table {
            family: NfFamily::INet,
            name: table_name.clone().into(),
            handle: None,
        }));

        wrapped_apply_ruleset(batch.to_nftables(), is_tokio)
            .await
            .unwrap();

        let current_ruleset = wrapped_get_current_ruleset(is_tokio).await.unwrap();
        let mut valid = false;

        for object in current_ruleset.objects.iter() {
            match object {
                NfObject::ListObject(object) => match object {
                    NfListObject::Table(table) if table.name == table_name => {
                        valid = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if !valid {
            panic!("Current ruleset did not contain expected table: {table_name}");
        }
    });
}

async fn wrapped_apply_ruleset(
    nftables: Nftables<'_>,
    is_tokio: bool,
) -> Result<(), NftablesError> {
    if is_tokio {
        apply_ruleset::<TokioProcess>(&nftables, None, None).await
    } else {
        apply_ruleset::<AsyncProcess>(&nftables, None, None).await
    }
}

async fn wrapped_get_current_ruleset(is_tokio: bool) -> Result<Nftables<'static>, NftablesError> {
    if is_tokio {
        get_current_ruleset::<TokioProcess>(None, None).await
    } else {
        get_current_ruleset::<AsyncProcess>(None, None).await
    }
}

fn run_test<F, Fut>(function: F)
where
    F: Fn(bool) -> Fut,
    Fut: Future<Output = ()> + Send,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(function(true));

    async_io::block_on(function(false));
}
