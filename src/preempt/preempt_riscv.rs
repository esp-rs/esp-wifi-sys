use esp32c3_hal::interrupt::TrapFrame;

#[derive(Debug, Default, Clone, Copy)]
pub struct Context {
    trap_frame: TrapFrame,
    pc: usize,
    _running: bool,
}

const STACK_SIZE: usize = 8192 * 2; // TODO how much is enough? would be better to have this per task
const MAX_TASK: usize = 3;

static mut TASK_STACK: [u8; STACK_SIZE * MAX_TASK] = [0u8; STACK_SIZE * MAX_TASK];

static mut FIRST_SWITCH: bool = true;

static mut TASK_TOP: usize = 0;

static mut CTX_NOW: usize = 0;

static mut CTX_TASKS: [Context; MAX_TASK] = [Context {
    trap_frame: TrapFrame {
        ra: 0,
        t0: 0,
        t1: 0,
        t2: 0,
        t3: 0,
        t4: 0,
        t5: 0,
        t6: 0,
        a0: 0,
        a1: 0,
        a2: 0,
        a3: 0,
        a4: 0,
        a5: 0,
        a6: 0,
        a7: 0,
        s0: 0,
        s1: 0,
        s2: 0,
        s3: 0,
        s4: 0,
        s5: 0,
        s6: 0,
        s7: 0,
        s8: 0,
        s9: 0,
        s10: 0,
        s11: 0,
        gp: 0,
        tp: 0,
        sp: 0,
    },
    pc: 0,
    _running: false,
}; MAX_TASK];

pub fn task_create(task: extern "C" fn()) -> usize {
    unsafe {
        let i = TASK_TOP;
        TASK_TOP += 1;
        CTX_TASKS[i].pc = task as usize;

        // stack must be aligned by 16
        let task_stack_ptr = &TASK_STACK as *const _ as usize
            + (STACK_SIZE as usize * i as usize)
            + STACK_SIZE as usize
            - 4;
        let stack_ptr = task_stack_ptr - (task_stack_ptr % 0x10);
        CTX_TASKS[i].trap_frame.sp = stack_ptr;

        CTX_NOW = i;
        i
    }
}

fn task_create_from_mepc(mepc: usize) -> usize {
    unsafe {
        let i = TASK_TOP;
        TASK_TOP += 1;
        CTX_TASKS[i].pc = mepc;
        CTX_NOW = i;
        i
    }
}

pub fn task_to_trap_frame(id: usize, trap_frame: &mut TrapFrame) -> usize {
    unsafe {
        trap_frame.ra = CTX_TASKS[id].trap_frame.ra;
        trap_frame.sp = CTX_TASKS[id].trap_frame.sp;
        trap_frame.a0 = CTX_TASKS[id].trap_frame.a0;
        trap_frame.a1 = CTX_TASKS[id].trap_frame.a1;
        trap_frame.a2 = CTX_TASKS[id].trap_frame.a2;
        trap_frame.a3 = CTX_TASKS[id].trap_frame.a3;
        trap_frame.a4 = CTX_TASKS[id].trap_frame.a4;
        trap_frame.a5 = CTX_TASKS[id].trap_frame.a5;
        trap_frame.a6 = CTX_TASKS[id].trap_frame.a6;
        trap_frame.a7 = CTX_TASKS[id].trap_frame.a7;
        trap_frame.t0 = CTX_TASKS[id].trap_frame.t0;
        trap_frame.t1 = CTX_TASKS[id].trap_frame.t1;
        trap_frame.t2 = CTX_TASKS[id].trap_frame.t2;
        trap_frame.t3 = CTX_TASKS[id].trap_frame.t3;
        trap_frame.t4 = CTX_TASKS[id].trap_frame.t4;
        trap_frame.t5 = CTX_TASKS[id].trap_frame.t5;
        trap_frame.t6 = CTX_TASKS[id].trap_frame.t6;
        trap_frame.s0 = CTX_TASKS[id].trap_frame.s0;
        trap_frame.s1 = CTX_TASKS[id].trap_frame.s1;
        trap_frame.s2 = CTX_TASKS[id].trap_frame.s2;
        trap_frame.s3 = CTX_TASKS[id].trap_frame.s3;
        trap_frame.s4 = CTX_TASKS[id].trap_frame.s4;
        trap_frame.s5 = CTX_TASKS[id].trap_frame.s5;
        trap_frame.s6 = CTX_TASKS[id].trap_frame.s6;
        trap_frame.s7 = CTX_TASKS[id].trap_frame.s7;
        trap_frame.s8 = CTX_TASKS[id].trap_frame.s8;
        trap_frame.s9 = CTX_TASKS[id].trap_frame.s9;
        trap_frame.s10 = CTX_TASKS[id].trap_frame.s10;
        trap_frame.s11 = CTX_TASKS[id].trap_frame.s11;
        trap_frame.gp = CTX_TASKS[id].trap_frame.gp;
        trap_frame.tp = CTX_TASKS[id].trap_frame.tp;

        CTX_TASKS[id].pc
    }
}

pub fn trap_frame_to_task(id: usize, pc: usize, trap_frame: &TrapFrame) {
    unsafe {
        CTX_TASKS[id].trap_frame.ra = trap_frame.ra;
        CTX_TASKS[id].trap_frame.sp = trap_frame.sp;
        CTX_TASKS[id].trap_frame.a0 = trap_frame.a0;
        CTX_TASKS[id].trap_frame.a1 = trap_frame.a1;
        CTX_TASKS[id].trap_frame.a2 = trap_frame.a2;
        CTX_TASKS[id].trap_frame.a3 = trap_frame.a3;
        CTX_TASKS[id].trap_frame.a4 = trap_frame.a4;
        CTX_TASKS[id].trap_frame.a5 = trap_frame.a5;
        CTX_TASKS[id].trap_frame.a6 = trap_frame.a6;
        CTX_TASKS[id].trap_frame.a7 = trap_frame.a7;
        CTX_TASKS[id].trap_frame.t0 = trap_frame.t0;
        CTX_TASKS[id].trap_frame.t1 = trap_frame.t1;
        CTX_TASKS[id].trap_frame.t2 = trap_frame.t2;
        CTX_TASKS[id].trap_frame.t3 = trap_frame.t3;
        CTX_TASKS[id].trap_frame.t4 = trap_frame.t4;
        CTX_TASKS[id].trap_frame.t5 = trap_frame.t5;
        CTX_TASKS[id].trap_frame.t6 = trap_frame.t6;
        CTX_TASKS[id].trap_frame.s0 = trap_frame.s0;
        CTX_TASKS[id].trap_frame.s1 = trap_frame.s1;
        CTX_TASKS[id].trap_frame.s2 = trap_frame.s2;
        CTX_TASKS[id].trap_frame.s3 = trap_frame.s3;
        CTX_TASKS[id].trap_frame.s4 = trap_frame.s4;
        CTX_TASKS[id].trap_frame.s5 = trap_frame.s5;
        CTX_TASKS[id].trap_frame.s6 = trap_frame.s6;
        CTX_TASKS[id].trap_frame.s7 = trap_frame.s7;
        CTX_TASKS[id].trap_frame.s8 = trap_frame.s8;
        CTX_TASKS[id].trap_frame.s9 = trap_frame.s9;
        CTX_TASKS[id].trap_frame.s10 = trap_frame.s10;
        CTX_TASKS[id].trap_frame.s11 = trap_frame.s11;
        CTX_TASKS[id].trap_frame.gp = trap_frame.gp;
        CTX_TASKS[id].trap_frame.tp = trap_frame.tp;

        CTX_TASKS[id].pc = pc;
    }
}

pub fn next_task() {
    unsafe {
        CTX_NOW = (CTX_NOW + 1) % TASK_TOP;
    }
}

pub fn task_switch(trap_frame: &mut TrapFrame) {
    unsafe {
        let old_mepc = riscv::register::mepc::read();

        if FIRST_SWITCH {
            FIRST_SWITCH = false;
            let main_task = task_create_from_mepc(old_mepc);
            CTX_NOW = main_task;
        }

        trap_frame_to_task(CTX_NOW, old_mepc, trap_frame);

        next_task();

        let new_pc = task_to_trap_frame(CTX_NOW, trap_frame);

        riscv::register::mepc::write(new_pc);

        // debug aid! remove when not needed anymore!!!!!
        // static mut CNT: u32 = 0;
        // if CTX_NOW == 0 {
        //     if CNT < 5_000 {
        //         CNT += 1;
        //     } else {
        //         CNT = 0;
        //         crate::println!("@@@ Task {} PC = {:x} {:x?}", CTX_NOW, new_pc, trap_frame);
        //     }
        // }
    }
}

pub fn current_task() -> usize {
    unsafe { CTX_NOW }
}
