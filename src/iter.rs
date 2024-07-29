#[derive(Default)]
pub enum EcsStep {
    #[default]
    Continue,
    Break,
}

#[derive(Default)]
pub enum EcsStepDestroy {
    #[default]
    Continue,
    Break,
    ContinueDestroy,
    BreakDestroy,
}

impl From<()> for EcsStep {
    #[inline(always)]
    fn from(_: ()) -> Self {
        EcsStep::Continue
    }
}

impl From<()> for EcsStepDestroy {
    #[inline(always)]
    fn from(_: ()) -> Self {
        EcsStepDestroy::Continue
    }
}

impl From<EcsStep> for EcsStepDestroy {
    #[inline(always)]
    fn from(step: EcsStep) -> Self {
        match step {
            EcsStep::Continue => EcsStepDestroy::Continue,
            EcsStep::Break => EcsStepDestroy::Break,
        }
    }
}
