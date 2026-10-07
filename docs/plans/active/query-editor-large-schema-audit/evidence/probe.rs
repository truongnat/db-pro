use db_pro_ui::{UiSchemaSummary,UiTableSummary,UiSchemaColumn};
use db_pro_ui::query::{SchemaCompletionProvider,CompletionContext,SchemaSymbolIndex};
use std::time::Instant;
use std::hint::black_box;
fn fixture(n: usize, cols: usize)->UiSchemaSummary {
 let tables:Vec<_>=(0..n).map(|i|UiTableSummary{schema:"public".into(),name:format!("table_{i:04}"),row_count:None,columns:(0..cols).map(|j|UiSchemaColumn{name:format!("column_{j:03}"),data_type:"text".into(),nullable:true,is_primary_key:j==0}).collect(),foreign_keys:vec![]}).collect();
 UiSchemaSummary{schemas:vec!["public".into()],tables:tables.iter().map(|t|t.name.clone()).collect(),table_details:tables,..Default::default()}
}
fn complete(schema:&UiSchemaSummary,before:&str,after:&str)->usize{
 let (_,items)=SchemaCompletionProvider::provide(&CompletionContext{text_before_cursor:before,text_after_cursor:after,cursor_offset:before.len(),active_schema:"public",schema_summary:schema,cached_tokens:None,is_sqlite:false,is_manual_trigger:true});black_box(items.len())
}
fn main(){
 for n in [50,500,1000] {
  let s=fixture(n,40);let t=Instant::now();let index=SchemaSymbolIndex::build(&s);println!("tables={n} columns={} index_ms={:.3}",n*40,t.elapsed().as_secs_f64()*1000.);black_box(&index);
  for (name,before,after) in [("qualified","SELECT t.col"," FROM public.table_0000 t"),("unscoped","SELECT col",""),("table-list","SELECT * FROM table_","")] {
   let mut times=vec![];let mut items=0;for _ in 0..15 {let t=Instant::now();items=complete(&s,before,after);times.push(t.elapsed().as_secs_f64()*1000.);}
   times.sort_by(|a,b|a.total_cmp(b));println!("tables={n} mode={name} items={items} median_ms={:.3} p95_ms={:.3}",times[7],times[14]);
  }
 }
 let s=fixture(500,40);
 let after=format!(" FROM public.table_0000 t; {}", "SELECT * FROM public.table_0001;\n".repeat(2000));
 let t=Instant::now();let count=complete(&s,"SELECT t.col",&after);println!("long_sql_bytes={} qualified_items={count} completion_ms={:.3}",after.len(),t.elapsed().as_secs_f64()*1000.);
 let result=db_pro_ui::UiQueryResult{columns:(0..20).map(|i|db_pro_ui::UiColumn{name:format!("c{i}"),data_type:"text".into(),nullable:true}).collect(),rows:(0..500).map(|_|(0..20).map(|_|db_pro_ui::UiCell::Text("x".repeat(8192))).collect()).collect(),row_count:500,duration_ms:0};
 let mut timings=vec![];for _ in 0..10{let t=Instant::now();let copy=result.clone();black_box(&copy);timings.push(t.elapsed().as_secs_f64()*1000.);drop(copy);}
 timings.sort_by(|a,b|a.total_cmp(b));println!("result_payload_mib=78.125 rows=500 columns=20 clone_median_ms={:.3} clone_max_ms={:.3}",timings[5],timings[9]);
 let r=std::panic::catch_unwind(||complete(&s,"SELECT tên",""));println!("utf8_completion_panics={}",r.is_err());
}
