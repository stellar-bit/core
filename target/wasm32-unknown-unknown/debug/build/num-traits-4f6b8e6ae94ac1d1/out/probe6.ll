; ModuleID = 'probe6.1a33c92f-cgu.0'
source_filename = "probe6.1a33c92f-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

@alloc_b19727876136523b0653010dd844c2ee = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/44cfafe2fafe816395d3acc434663a45d5178c41/library/core/src/num/mod.rs" }>, align 1
@alloc_00795e26fe556e999f629d259155e0d3 = private unnamed_addr constant <{ ptr, [12 x i8] }> <{ ptr @alloc_b19727876136523b0653010dd844c2ee, [12 x i8] c"K\00\00\00/\04\00\00\05\00\00\00" }>, align 4
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe6::probe
; Function Attrs: nounwind
define hidden void @_ZN6probe65probe17hf6d29890137e61bdE() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h5674d486976426a6E.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h9f9a3962ff65bf62E(ptr align 1 @str.0, i32 25, ptr align 4 @alloc_00795e26fe556e999f629d259155e0d3) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h5674d486976426a6E.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind readnone willreturn
declare hidden i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn nounwind
declare dso_local void @_ZN4core9panicking5panic17h9f9a3962ff65bf62E(ptr align 1, i32, ptr align 4) unnamed_addr #2

attributes #0 = { nounwind "target-cpu"="generic" }
attributes #1 = { nocallback nofree nosync nounwind readnone willreturn }
attributes #2 = { cold noinline noreturn nounwind "target-cpu"="generic" }
attributes #3 = { noreturn nounwind }
