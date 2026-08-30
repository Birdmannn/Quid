import Image from "next/image";
import stellarIcon from "../../../../../../public/quest-detail/stellar-icon.png";
import { FieldLabel, NumberField, SelectField } from "../fields";
import {
  AVAILABLE_WALLET_BALANCE,
  rewardCalculations,
  type RewardsData,
} from "../types";

const REWARD_METHOD_OPTIONS = ["Selected winners", "All participants"];

function XlmIcon() {
  return <Image src={stellarIcon} alt="" width={14} height={14} className="h-3.5 w-3.5" />;
}

export default function RewardsStep({
  data,
  onChange,
}: {
  data: RewardsData;
  onChange: (patch: Partial<RewardsData>) => void;
}) {
  const { platformFee, networkFee, totalRequired, remaining, rewardPerWinner } =
    rewardCalculations(data);

  return (
    <div className="flex flex-col gap-6">
      <p className="text-sm text-white/50">
        Set how rewards are split. Any unused rewards stay in the funding
        wallet. Fees shown are just estimates until they&apos;re actually
        distributed.
      </p>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div>
          <FieldLabel htmlFor="reward-method">Reward method</FieldLabel>
          <SelectField
            id="reward-method"
            value={data.rewardMethod}
            onChange={(rewardMethod) => onChange({ rewardMethod })}
            options={REWARD_METHOD_OPTIONS}
          />
        </div>
        <div>
          <FieldLabel htmlFor="total-reward-budget">
            Total reward budget
          </FieldLabel>
          <NumberField
            id="total-reward-budget"
            value={data.totalRewardBudget}
            onChange={(totalRewardBudget) => onChange({ totalRewardBudget })}
            min={0}
            prefixIcon={<XlmIcon />}
          />
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div>
          <FieldLabel htmlFor="number-of-winners">Number of winners</FieldLabel>
          <NumberField
            id="number-of-winners"
            value={data.numberOfWinners}
            onChange={(numberOfWinners) => onChange({ numberOfWinners })}
            min={1}
          />
        </div>
        <div>
          <FieldLabel htmlFor="reward-per-winner">Reward per winner</FieldLabel>
          <div className="flex h-[42px] items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.02] px-3 text-sm text-white/70">
            <XlmIcon />
            {rewardPerWinner.toLocaleString(undefined, {
              maximumFractionDigits: 2,
            })}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 rounded-xl border border-white/10 bg-[#100D1C]/60 p-4 sm:grid-cols-3">
        <div>
          <p className="text-xs text-white/40">Total required balance</p>
          <p className="mt-1 text-lg font-semibold text-white">
            {totalRequired.toLocaleString(undefined, { maximumFractionDigits: 2 })}{" "}
            XLM
          </p>
        </div>
        <div>
          <p className="text-xs text-white/40">Available wallet balance</p>
          <p className="mt-1 text-lg font-semibold text-white">
            {AVAILABLE_WALLET_BALANCE.toLocaleString()} XLM
          </p>
        </div>
        <div>
          <p className="text-xs text-white/40">Remaining balance</p>
          <p
            className={`mt-1 text-lg font-semibold ${remaining < 0 ? "text-red-400" : "text-white"}`}
          >
            {remaining.toLocaleString(undefined, { maximumFractionDigits: 2 })}{" "}
            XLM
          </p>
        </div>
      </div>

      <p className="text-xs text-white/40">
        Platform fee {platformFee.toLocaleString(undefined, { maximumFractionDigits: 2 })}{" "}
        XLM &nbsp;•&nbsp; Network fee estimate {networkFee} XLM
      </p>
      {remaining < 0 ? (
        <p className="text-xs text-red-400">
          Your reward budget exceeds your available wallet balance.
        </p>
      ) : null}
    </div>
  );
}

export function isRewardsStepValid(data: RewardsData): boolean {
  const { remaining } = rewardCalculations(data);
  return data.totalRewardBudget > 0 && data.numberOfWinners > 0 && remaining >= 0;
}
