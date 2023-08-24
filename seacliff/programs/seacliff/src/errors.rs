use anchor_lang::error_code;

#[error_code]
pub enum ContractError {
    /// Cannot call new on (initialize) an already initialized contract.
    ContractAlreadyInitialized,
    /// Cannot perform action on a contract that is not active.
    ContractNotActive,
    /// Cannot initialize a goal that is less than the MINIMUM_GOAL constant.
    GoalLessThanMinimum,
    /// Cannot initialize a refund bonus that is less than the MINIMUM_REFUND_BONUS constant.
    RefundBonusLessThanMinimum,
    /// Cannot initialize a lifespan that is less than the MINIMUM_LIFESPAN constant.
    LifespanLessThanMinimum,
    /// Cannot initialize a lifespan that is greater than the MAXIMUM_LIFESPAN constant.
    LifespanLGreaterThanMaximum,
    /// Cannot initialize a contract that is already self-sufficient.
    GoalLessThanRefundBonus,
    /// The proposer does not have enough lamports to cover the refund bonus.
    NotEnoughLamportsForRefundBonus,
    /// The operation would cause an overflow.
    ArithmeticOverflow,
    /// Cannot pledge to a contract that has passed its deadline.
    DeadlinePassed,
    /// Cannot pledge less than the MINIMUM_Pledge constant.
    PledgeLessThanMinimum,
    /// The backer does not have enough lamports to cover the pledge.
    NotEnoughLamportsForPledge,
    /// Cannot close a contract that has not passed its deadline.
    CloseBeforeDeadline,
    /// Cannot redeem refund bonus on active contract or funded contract.
    ContractNotAvailableForRedeem,
}
