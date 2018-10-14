module Adjustments.Messages exposing (Msg(..))

import Adjustments.Models exposing (Adjustment)
import Http


type Msg
    = TitleUpdated String
    | AmountUpdated String
    | SaveAdjustment
    | DeleteAdjustment
