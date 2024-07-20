pub enum EcsStep {
    Continue,
    Break,
}

pub enum EcsStepDestroy {
    Continue,
    Break,
    ContinueDestroy,
    BreakDestroy,
}

impl Default for EcsStep {
    fn default() -> Self {
        EcsStep::Continue
    }
}

impl Default for EcsStepDestroy {
    fn default() -> Self {
        EcsStepDestroy::Continue
    }
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
